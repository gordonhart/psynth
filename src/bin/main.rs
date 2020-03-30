use anyhow::{anyhow, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use psynth::{generators, filters, consumers, FilterComposable, Sample, Observer};


fn main() -> Result<()> {
    let host = cpal::default_host();
    let output_device = host
        .default_output_device()
        .ok_or_else(|| anyhow!("missing default output device"))?;
    let config_supported = output_device.default_output_config()?;
    let config: cpal::StreamConfig = config_supported.into();
    println!("default config: {:?}", config);

    let channels = config.channels as usize;
    // let mut gen: generators::Generator = generators::flat(&config, 440.0);
    // let mut gen: psynth::Generator = generators::sub_server(0)?;
    let mut gen: psynth::Generator =
        // generators::sine(&config, 440.0)
        generators::microphone(&host, &config)
        .compose(filters::ramp_up(&config, 0.01))
        // .compose(filters::gain(0.1))
        // .compose(filters::ramp_down(&config, 1.0, 0.01))
        // .compose(filters::comb(&config, 0.05, 0.25))
        .compose(filters::comb(&config, 0.15, 0.25))
        .compose(filters::comb(&config, 0.25, 0.25))
        .compose(filters::comb(&config, 0.50, 0.25))
        .compose(filters::comb(&config, 0.2, 0.5))
        .compose(filters::comb(&config, 1.0, 0.1))
        .compose(filters::warble(&config, 1.0))
        ;

    let _observers: Vec<Box<dyn Observer + Send>> = vec![Box::new(std::io::stdout())];
    // let mut consumer = consumers::write_output_stream_mono_with_observers(channels, observers);
    let mut consumer = consumers::write_output_stream_mono(channels);

    let output_stream = output_device.build_output_stream(
        &config,
        move |obuf: &mut [Sample]| consumer(&mut gen, obuf),
        move |err| panic!("audio stream error: {:?}", err),
    )?;
    output_stream.play()?;
    std::thread::sleep(std::time::Duration::from_secs(60));

    Ok(())
}
