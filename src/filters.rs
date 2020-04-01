use std::collections::VecDeque;
use std::f32::consts::PI;

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
