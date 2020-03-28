use anyhow::{anyhow, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use psynth::generators::{flat, server};


fn main() -> Result<()> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or_else(|| anyhow!("missing default output device"))?;
    let config_supported = device.default_output_config()?;
    let config: cpal::StreamConfig = config_supported.into();
    println!("default config: {:?}", config);

    let stream = device.build_output_stream(
        &config,
        // flat(&config, 1000.0),
        server(&config)?,
        move |err| panic!("audio stream error: {:?}", err),
    )?;
    stream.play()?;
    std::thread::sleep(std::time::Duration::from_millis(1_000_000));

    Ok(())
}
