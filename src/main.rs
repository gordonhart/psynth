use anyhow::{anyhow, Result};
use cpal::traits::{DeviceTrait, HostTrait};


fn main() -> Result<()> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or_else(|| anyhow!("missing default output device"))?;
    let config = device.default_output_config()?;
    println!("supported: {:?}", device.supported_output_configs()?.collect::<Vec<cpal::SupportedStreamConfigRange>>());
    println!("default: {:?}", config);

    Ok(())
}
