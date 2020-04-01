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
    let generator: psynth::Generator =
        // generators::sine(&config, 440.0)
        /*
        generators::sine(&config, controls::TimedSawtoothPot {
            low: 440.0,
            high: 1000.0,
            period: 1.0,
        })
        */
        // generators::sine(&config, controls::SinePot::new(&config, 0.25, 1000.0, 440.0))
        generators::sine(&config, controls::StdinPot::default())
        // generators::microphone(&host, &config)
        .compose(filters::ramp_up(&config, 0.01))
        // .compose(filters::gain(controls::TimedSawtoothPot::default()))
        .compose(filters::gain(0.1))
        // .compose(filters::ramp_down(&config, 1.0, 0.01))
        // .compose(filters::reverb(&config, 0.0, 0.0))
        // .compose(filters::warble(&config, 1.0))
        ;

    let observers: Vec<Box<dyn Observer + Send>> = vec![
        // Box::new(std::io::stdout())
    ];
    let mut consumer = consumers::MonoConsumer::new(channels)
        .bind(generator)
        .bind_observers(observers)
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
