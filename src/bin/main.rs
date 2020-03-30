use anyhow::{anyhow, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use psynth::{generators, filters, consumers, FilterComposable, Sample, Observer};


fn main() -> Result<()> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or_else(|| anyhow!("missing default output device"))?;
    let config_supported = device.default_output_config()?;
    let config: cpal::StreamConfig = config_supported.into();
    println!("default config: {:?}", config);

    let channels = config.channels as usize;
    // let mut gen: generators::Generator = generators::flat(&config, 440.0);
    // let mut gen: psynth::Generator = generators::sub_server(0)?;
    let mut gen: psynth::Generator = generators::sine(&config, 440.0);
        // .compose(filters::warble(&config, 1.0));
        // .compose(filters::warble(&config, 3.0))
        // .compose(filters::warble(&config, 4.0));

    let observers: Vec<Box<dyn Observer + Send>> = vec![Box::new(std::io::stdout())];
    let mut consumer = consumers::write_output_stream_mono_with_observers(channels, observers);

    let stream = device.build_output_stream(
        &config,
        // move |obuf: &mut [Sample]| consumers::write_output_stream_mono(channels)(&mut gen, obuf),
        move |obuf: &mut [Sample]| consumer(&mut gen, obuf),
        move |err| panic!("audio stream error: {:?}", err),
    )?;
    stream.play()?;
    std::thread::sleep(std::time::Duration::from_millis(1_000_000));

    Ok(())
}
