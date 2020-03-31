use std::collections::VecDeque;
use std::f32::consts::PI;

use crate::{
    Sample,
    Filter,
    Generator,
    Pot,
    // FilterComposable,
};


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
pub fn warble(
    config: &cpal::StreamConfig,
    period: f32,
) -> Filter {

    // period spans (sample_rate * period) samples
    let sample_rate = config.sample_rate.0 as f32;
    let mut x = 0f32;

    Box::new(move |sample: Sample| {
        x = (x + 1.0) % (sample_rate * period);
        let amplitude_modulation = ((2.0 * PI * x) / (sample_rate * period)).sin();
        sample * amplitude_modulation
    })
}


/// Scale the signal by the provided scale factor with clipping at `[-1, 1]`.
pub fn gain<P>(scale_factor: P) -> Filter
where
    P: Pot<f32> + 'static
{
    Box::new(move |sample: Sample| {
        let val = sample * scale_factor.read();
        if val > 1.0 {
            1.0
        } else if val < -1.0 {
            -1.0
        } else {
            val
        }
    })
}


/// Ramp gain from zero to one over the specified number of seconds.
pub fn ramp_up(config: &cpal::StreamConfig, ramp_secs: f32) -> Filter {
    let sample_rate = config.sample_rate.0 as f32;
    let ramp_steps: f32 = sample_rate * ramp_secs;
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
pub fn ramp_down(config: &cpal::StreamConfig, cliff_secs: f32, ramp_secs: f32) -> Filter {
    let sample_rate = config.sample_rate.0 as f32;
    let cliff_steps = sample_rate * cliff_secs;
    let ramp_steps: f32 = sample_rate * ramp_secs;
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
    config: &cpal::StreamConfig,
    delay_secs: f32,
    decay_factor: f32, // decay this much in amplitude each time the delay is repeated
    direction: CombDirection,
) -> Filter {
    let sample_rate = config.sample_rate.0 as f32;

    // number of samples until a given sample echoes
    let k = delay_secs * sample_rate;
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
    config: &cpal::StreamConfig,
    delay_secs: f32,
    decay_factor: f32,
) -> Filter {
    let mut comb_forward = comb(config, delay_secs, decay_factor, CombDirection::FeedForward);
    let mut comb_back = comb(config, delay_secs, -decay_factor, CombDirection::FeedBack);
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
    config: &cpal::StreamConfig,
    _delay_secs: f32,
    _decay_factor: f32,
) -> Filter {
    let mut combs = parallel(vec![
        comb(&config, 0.09999, 0.742, CombDirection::FeedBack),
        comb(&config, 0.10414, 0.733, CombDirection::FeedBack),
        comb(&config, 0.11248, 0.715, CombDirection::FeedBack),
        comb(&config, 0.12085, 0.697, CombDirection::FeedBack),
    ]);
    let mut all_pass_a = all_pass(&config, 0.02189, 0.7);
    let mut all_pass_b = all_pass(&config, 0.00702, 0.7);
    Box::new(move |sample: Sample| all_pass_b(all_pass_a(combs(sample))))
}
