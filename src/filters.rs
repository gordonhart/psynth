use std::collections::VecDeque;
use std::f32::consts::PI;

use crate::{Filter, Generator, FilterComposable};


/// Apply the given `Filter` to the given `Generator` and return a `Generator` interface.
pub fn compose(mut generator: Generator, mut filter: Filter) -> Generator {
    Box::new(move || filter(&mut generator))
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

    Box::new(move |generator: &mut Generator| {
        x = (x + 1.0) % (sample_rate * period);
        let original_value = generator();
        let amplitude_modulation = ((2.0 * PI * x) / (sample_rate * period)).sin();
        original_value * amplitude_modulation
    })
}


/// Scale the signal by the provided scale factor with clipping at `[-1, 1]`.
pub fn gain(scale_factor: f32) -> Filter {
    Box::new(move |generator: &mut Generator| {
        let val = generator() * scale_factor;
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

    Box::new(move |generator: &mut Generator| {
        if ramp_i < ramp_steps {
            ramp_i += 1.0;
        }
        generator() * (ramp_i / ramp_steps)
    })
}


/// Ramp the signal to zero after the provided time has elapsed and over the specified number of
/// seconds.
pub fn ramp_down(config: &cpal::StreamConfig, cliff_secs: f32, ramp_secs: f32) -> Filter {
    let sample_rate = config.sample_rate.0 as f32;
    let cliff_steps = sample_rate * cliff_secs;
    let ramp_steps: f32 = sample_rate * ramp_secs;
    let mut ramp_i = 0f32;

    Box::new(move |generator: &mut Generator| {
        let mut val = generator();
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

    Box::new(move |generator: &mut Generator| {
        let mut val = generator();
        match direction {
            CombDirection::FeedForward => {
                buf.push_back(val);
                val += decay_factor * buf.pop_front().unwrap_or(0.0);
            },
            CombDirection::FeedBack => {
                val += decay_factor * buf.pop_front().unwrap_or(0.0);
                buf.push_back(val);
            },
        };
        val
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

    Box::new(move |generator: &mut Generator| {
        let val = comb_forward(generator);
        let mut fakegen: Generator = Box::new(move || val);
        comb_back(&mut fakegen)
        // TODO: chain filters together within a filter without needing to allocate
        /* maybe something like this:
        generator
            .compose_ref(&mut comb_forward)
            .compose_ref(&mut comb_back)
            ()
        */
    })
}


pub fn reverb(
    config: &cpal::StreamConfig,
    delay_secs: f32,
    decay_factor: f32,
) -> Filter {
    unimplemented!("{:?}, {:?}, {:?}", config, delay_secs, decay_factor);
}
