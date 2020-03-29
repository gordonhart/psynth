use std::f32::consts::PI;

use crate::{Filter, Generator};


/// Apply the given `Filter` to the given `Generator` and return a `Generator` interface.
pub fn compose(mut generator: Generator, mut filter: Filter) -> Generator {
    Box::new(move || filter(&mut generator))
}


/// Sinusoidally attenuate the volume of the output with the provided period.
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
        let amplitude_modulation = {
            let raw_mod = ((2.0 * PI * x) / (sample_rate * period)).sin();
            (raw_mod + 1.0) / 2.0
        };
        original_value * amplitude_modulation
    })
}
