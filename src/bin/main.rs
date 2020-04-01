use anyhow::{anyhow, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use psynth::{
    generators,
    filters,
    consumers,
    controls,
    Consumer,
    FilterComposable,
    Sample,
    Observer,
};


fn main() -> Result<()> {
    let host = cpal::default_host();
    let output_device = host
        .default_output_device()
        .ok_or_else(|| anyhow!("missing default output device"))?;
    let config_supported = output_device.default_output_config()?;
    let config: cpal::StreamConfig = config_supported.into();
    println!("default config: {:?}", config);

    let channels = config.channels as usize;
    let rate: u32 = config.sample_rate.0;
    let left_generator = generators::sine(rate, 200.0)
        .compose(filters::gain(controls::sine_pot(rate, 1.0 / 3.0, 0.0, 1.0)))
        .compose(filters::gain(0.5));
    let right_generator = generators::sine(rate, 250.0)
        .compose(filters::gain(controls::sine_pot(rate, 1.0 / 2.0, 0.0, 1.0)))
        .compose(filters::gain(0.5));
    /*
    let generator: psynth::Generator =
        generators::multi(vec![
            generators::microphone(&host, &config)
                .compose(filters::reverb(rate, 0.0, 0.0)),
            generators::sine(rate, 200.0)
                .compose(filters::gain(controls::sine_pot(rate, 1.0 / 3.0, 0.0, 1.0)))
                .compose(filters::gain(0.5)),
            generators::sine(rate, controls::StdinPot::default())
                .compose(filters::gain(controls::sine_pot(rate, 1.0 / 4.0, 0.0, 1.0)))])
        .compose(filters::ramp_up(rate, 0.01))
        .compose(filters::reverb(rate, 0.0, 0.0))
        .compose(filters::gain(0.025))
        ;
        */

    let observers: Vec<Box<dyn Observer + Send>> = vec![
        // Box::new(std::io::stdout())
    ];
    /*
    let mut consumer = consumers::MonoConsumer::new(channels)
        .bind(generator)
        .bind_observers(observers)
        ;
    */
    let mut consumer = consumers::StereoConsumer::default()
        .bind(left_generator, right_generator)
        ;

    let output_stream = output_device.build_output_stream(
        &config,
        move |obuf: &mut [Sample]| consumer.fill(obuf),
        move |err| panic!("audio stream error: {:?}", err),
    )?;
    output_stream.play()?;
    std::thread::sleep(std::time::Duration::from_secs(600));

    Ok(())
}
