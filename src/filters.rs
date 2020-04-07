use std::collections::VecDeque;
use std::f32::consts::PI;
use std::sync::{Arc, Mutex};

use crate::{Sample, Filter, Generator, Pot};


/// Apply the given `Filter` to the given `Generator` and return a `Generator` interface.
///
/// Consumes both of the provided arguments.
pub fn compose(mut generator: Generator, mut filter: Filter) -> Generator {
    Box::new(move || filter(generator()))
}


/// Sinusoidally attenuate the volume of the output with the provided period.
///
/// Note that this travels through the entire range of the sinusoid (-1, 1) on a given period,
/// meaning that the heard effect here is a warbling with period double that of the provided
/// `period`.
// TODO: deprecate in favor of pot-actuated gain?
pub fn warble(sample_rate: u32, period: f32) -> Filter {

    // period spans (sample_rate * period) samples
    let rate = sample_rate as f32;
    let mut x = 0f32;

    Box::new(move |sample: Sample| {
        x = (x + 1.0) % (rate * period);
        let amplitude_modulation = ((2.0 * PI * x) / (rate * period)).sin();
        sample * amplitude_modulation
    })
}


/// Scale the signal by the provided scale factor.
///
/// No clipping is performed.
pub fn gain<P>(scale_factor: P) -> Filter
where
    P: Pot<f32> + 'static,
{
    Box::new(move |sample: Sample| sample * scale_factor.read())
}


/// Clip the waveform by thresholding to the range `[low, high]`.
///
/// Example usage would be for an 'overdrive' effect that clips at `[-1.0, 1.0]`.
pub fn clip<P>(low: P, high: P) -> Filter
where
    P: Pot<f32> + 'static,
{
    Box::new(move |sample: Sample| {
        let lo = low.read();
        let hi = high.read();
        if sample < lo {
            lo
        } else if sample > hi {
            hi
        } else {
            sample
        }
    })
}


/// Ramp gain from zero to one over the specified number of seconds.
pub fn ramp_up(sample_rate: u32, ramp_secs: f32) -> Filter {
    let rate = sample_rate as f32;
    let ramp_steps: f32 = rate * ramp_secs;
    let mut ramp_i = 0f32;

    Box::new(move |sample: Sample| {
        if ramp_i < ramp_steps {
            ramp_i += 1.0;
        }
        sample * (ramp_i / ramp_steps)
    })
}


/// Ramp the signal to zero after the provided time has elapsed and over the specified number of
/// seconds.
pub fn ramp_down(sample_rate: u32, cliff_secs: f32, ramp_secs: f32) -> Filter {
    let rate = sample_rate as f32;
    let cliff_steps = rate * cliff_secs;
    let ramp_steps: f32 = rate * ramp_secs;
    let mut ramp_i = 0f32;

    Box::new(move |sample: Sample| {
        let mut val = sample;
        if ramp_i < cliff_steps + ramp_steps {
            ramp_i += 1.0;
        }
        if ramp_i >= cliff_steps {
            val *= 1.0 - ((ramp_i - cliff_steps) / ramp_steps);
        }
        val
    })
}


pub enum CombDirection {
    FeedForward,
    FeedBack,
}
/// [Comb](https://en.wikipedia.org/wiki/Comb_filter) echo filter.
///
/// Feedforward difference equation:
///
/// ```text
/// y(t) = x(t) + a * x(t - k)
/// ```
///
/// Feedback difference equation:
///
/// ```text
/// y(t) = x(t) + a * y(t - k)
/// ```
pub fn comb(
    sample_rate: u32,
    delay_secs: f32,
    decay_factor: f32, // decay this much in amplitude each time the delay is repeated
    direction: CombDirection,
) -> Filter {
    let rate = sample_rate as f32;

    // number of samples until a given sample echoes
    let k = delay_secs * rate;
    let bufsize = k as usize;
    let mut buf: VecDeque<f32> = VecDeque::from(vec![0.0; bufsize]);

    Box::new(move |sample: Sample| {
        match direction {
            CombDirection::FeedForward => {
                buf.push_back(sample);
                sample + decay_factor * buf.pop_front().unwrap_or(0.0)
            },
            CombDirection::FeedBack => {
                let out = sample + decay_factor * buf.pop_front().unwrap_or(0.0);
                buf.push_back(out);
                out
            },
        }
    })
}


/// All-pass filter implementation using two Comb filters (FeedForward, FeedBack) of equal delay
/// and opposite decay in series.
pub fn all_pass(
    sample_rate: u32,
    delay_secs: f32,
    decay_factor: f32,
) -> Filter {
    let mut comb_forward = comb(sample_rate, delay_secs, decay_factor, CombDirection::FeedForward);
    let mut comb_back = comb(sample_rate, delay_secs, -decay_factor, CombDirection::FeedBack);
    Box::new(move |sample: Sample| comb_back(comb_forward(sample))) 
}


/// Apply the provided `Filter`s and sum the results.
pub fn parallel(mut filters: Vec<Filter>) -> Filter {
    Box::new(move |sample: Sample| {
        let mut out = 0f32;
        for filter in filters.iter_mut() {
            out += filter(sample);
        }
        out
    })
}


// TODO: not hardcode values, actually used provided params
pub fn reverb(
    sample_rate: u32,
    _delay_secs: f32,
    _decay_factor: f32,
) -> Filter {
    let mut combs = parallel(vec![
        comb(sample_rate, 0.09999, 0.742, CombDirection::FeedBack),
        comb(sample_rate, 0.10414, 0.733, CombDirection::FeedBack),
        comb(sample_rate, 0.11248, 0.715, CombDirection::FeedBack),
        comb(sample_rate, 0.12085, 0.697, CombDirection::FeedBack),
    ]);
    let mut all_pass_a = all_pass(sample_rate, 0.02189, 0.7);
    let mut all_pass_b = all_pass(sample_rate, 0.00702, 0.7);
    Box::new(move |sample: Sample| all_pass_b(all_pass_a(combs(sample))))
}


/// Offset the provided value by the value retrieved from the held potentiometer.
///
/// Useful primarily in composing `Generator`s and `Filter`s as `Pot`s.
pub fn offset<P>(offset_pot: P) -> Filter
where
    P: Pot<Sample> + 'static,
{
    Box::new(move |sample: Sample| sample + offset_pot.read())
}


/// Apply a recursive filter as described in Chapter 19 of
/// [_The Scientist and Engineer's Guide to Digital Signal Processing_](https://www.analog.com/media/en/technical-documentation/dsp-book/dsp_book_Ch19.pdf).
///
/// This is somewhat painful to work with with real `Pot`s. When actual dynamic `Pot`
/// implementations are going to be used, look into using `recursive_helper` (private).
pub fn recursive(
    a_coeffs: Vec<Box<dyn Pot<f64>>>,
    b_coeffs: Vec<Box<dyn Pot<f64>>>,
) -> Filter {
    let mut recurse = recursive_helper(a_coeffs.len(), b_coeffs.len());
    Box::new(move |sample: Sample| {
        let a_coeffs_real = a_coeffs.iter().map(|a| a.read()).collect::<Vec<f64>>();
        let b_coeffs_real = b_coeffs.iter().map(|b| b.read()).collect::<Vec<f64>>();
        recurse(sample, a_coeffs_real.as_slice(), b_coeffs_real.as_slice())
    })
}


// less difficult to work with than `recursive` for dynamic pots, but serves the same function
fn recursive_helper(
    a_coeffs_len: usize,
    b_coeffs_len: usize,
) -> impl FnMut(Sample, &[f64], &[f64]) -> Sample
{
    let mut samples: VecDeque<f64> = VecDeque::from(vec![0.0; a_coeffs_len]);
    let mut outputs: VecDeque<f64> = VecDeque::from(vec![0.0; b_coeffs_len]);
    move |sample: Sample, a_coeffs: &[f64], b_coeffs: &[f64]| {
        samples.pop_back();
        samples.push_front(sample.into());
        let output = {
            let a_sum = samples.iter().zip(a_coeffs.iter()).fold(0.0, |acc, (s, a)| acc + (s * a));
            let b_sum = outputs.iter().zip(b_coeffs.iter()).fold(0.0, |acc, (o, b)| acc + (o * b));
            a_sum + b_sum
        };
        outputs.pop_back();
        outputs.push_front(output);
        output as f32
    }
}


/// Single-pole low-pass RC filter as described by Equation 19-2.
///
/// The value for `x` should be on `[0,1]`.
pub fn single_pole_low_pass<P>(x: P) -> Filter
where
    P: Pot<f64> + 'static,
{
    let mut recurse = recursive_helper(1, 1);
    Box::new(move |sample: Sample| {
        let this_x = x.read();
        recurse(sample, &[1.0 - this_x], &[this_x])
    })
}


/// Single-pole low-pass RC filter as described by Equation 19-3.
///
/// The value for `x` should be on `[0,1]`.
pub fn single_pole_high_pass<P>(x: P) -> Filter
where
    P: Pot<f64> + 'static,
{
    let mut recurse = recursive_helper(2, 1);
    Box::new(move |sample: Sample| {
        let this_x = x.read();
        recurse(
            sample,
            &[(1.0 + this_x) / 2.0, - (1.0 + this_x) / 2.0],
            &[this_x],
        )
    })
}


/// Four single-pole low-pass filters stacked in series to achieve a more ideal low-pass effect.
/// Equation 19-6 in the book. The value for`x` should be on `[0,1]`.
pub fn four_stage_low_pass<P>(x: P) -> Filter
where
    P: Pot<f64> + 'static,
{
    let x_arc = Arc::new(Mutex::new(x));
    let mut fs: Vec<Filter> = (0..4).map(|_| single_pole_low_pass(x_arc.clone())).collect();
    Box::new(move |sample: Sample| fs.iter_mut().fold(sample, |acc, f| f(acc)))
}


// internal helper function to generate r,k from f,bw as used by band_pass and notch filters
fn r_and_k_from_f_and_bw(f: f64, bw: f64) -> (f64, f64) {
    let pix2 = std::f64::consts::PI * 2.0;
    let r = 1.0 - 3.0 * bw;
    let k = {
        let numer = 1.0 - (2.0 * r * (pix2 * f).cos()) + r.powi(2);
        let denom = 2.0 - 2.0 * (pix2 * f).cos();
        numer / denom
    };
    (r, k)
}

/// Band pass filter that passes frequencies near the `center_frequency` falling off sharply
/// outside of the `band_width`.
///
/// Implementation of [Equation 19-7](https://www.analog.com/media/en/technical-documentation/dsp-book/dsp_book_Ch19.pdf).
///
/// Note that values for `center_frequency` near zero cause numerical instability.
pub fn band_pass<P1, P2>(
    sample_rate: u32,
    center_frequency: P1,
    band_width: P2,
) -> Filter
where
    P1: Pot<f64> + 'static,
    P2: Pot<f64> + 'static,
{
    let mut recurse = recursive_helper(3, 2);
    let pix2 = std::f64::consts::PI * 2.0;
    Box::new(move |sample: Sample| {
        let f_frac = center_frequency.read() / (sample_rate as f64);
        let bw_frac = band_width.read() / (sample_rate as f64);
        let (r, k) = r_and_k_from_f_and_bw(f_frac, bw_frac);
        recurse(
            sample,
            &[
                1.0 - k,
                2.0 * (k - r) * (pix2 * f_frac).cos(),
                r.powi(2) - k,
            ],
            &[
                2.0 * r * (pix2 * f_frac).cos(),
                - r.powi(2),
            ],
        )
    })
}


/// The opposite of `band_pass`, `notch` passes all but those frequencies near `center_frequency`
/// and within the `band_width`.
///
/// Described by equation 19-8 in the book.
pub fn notch<P1, P2>(
    sample_rate: u32,
    center_frequency: P1,
    band_width: P2,
) -> Filter
where
    P1: Pot<f64> + 'static,
    P2: Pot<f64> + 'static,
{
    let mut recurse = recursive_helper(3, 2);
    let pix2 = std::f64::consts::PI * 2.0;
    Box::new(move |sample: Sample| {
        let f_frac = center_frequency.read() / (sample_rate as f64);
        let bw_frac = band_width.read() / (sample_rate as f64);
        let (r, k) = r_and_k_from_f_and_bw(f_frac, bw_frac);
        recurse(
            sample,
            &[
                k,
                - 2.0 * k * (pix2 * f_frac).cos(),
                k,
            ],
            &[
                2.0 * r * (pix2 * f_frac).cos(),
                - r.powi(2),
            ],
        )
    })
}


// TODO: https://www.dsprelated.com/freebooks/filters/Elementary_Audio_Digital_Filters.html
// https://www.analog.com/media/en/technical-documentation/dsp-book/dsp_book_Ch19.pdf
pub fn equalizer<F>(_eq_function: F) -> Filter
where
    F: Fn(f32) -> f32 + 'static,
{
    unimplemented!()
}
