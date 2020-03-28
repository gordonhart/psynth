use anyhow::{anyhow, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use psynth::tones::write_flat;


fn main() -> Result<()> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or_else(|| anyhow!("missing default output device"))?;
    let config_supported = device.default_output_config()?;
    let config: cpal::StreamConfig = config_supported.into();
    println!("default: {:?}", config);

    let stream = device.build_output_stream(
        &config,
        write_flat(&config, 1000.0),
        move |err| {
            panic!("audio stream error: {:?}", err);
        },
    )?;
    stream.play()?;
    std::thread::sleep(std::time::Duration::from_millis(1000));

    Ok(())
}
