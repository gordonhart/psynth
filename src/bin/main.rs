use anyhow::{anyhow, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

#[allow(unused_imports)]
use psynth::{
    generator,
    filter,
    consumer,
    Pot,
    control,
    sampling,
    Consumer,
    FilterComposable,
    Sample,
    music::notes,
    // devices,
};


fn main() -> Result<()> {
    let host = cpal::default_host();
    let output_device = host
        .default_output_device()
        .ok_or_else(|| anyhow!("missing default output device"))?;

    // hardcode to match WAV files, not the best solution but OK for now
    let config = cpal::StreamConfig { channels: 2, sample_rate: cpal::SampleRate(44100) };
    println!("config: {:?}", config);

    let channels = config.channels as usize;
    let rate: u32 = config.sample_rate.0;

    let (l, r) = control::mux::balance(
        control::pot::sine_pot(rate, 4.0, -1.0, 1.0),
        generator::sine(rate, 440.0).compose(filter::gain(0.1)),
        generator::sine(rate, 330.0).compose(filter::gain(0.1)),
    );
    let mut consumer = consumer::StereoConsumer::new(channels).bind(l, r);

    let output_stream = output_device.build_output_stream(
        &config,
        move |obuf: &mut [Sample]| consumer.fill(obuf),
        move |err| panic!("audio stream error: {:?}", err),
    )?;
    output_stream.play()?;

    // time out after 60 seconds
    std::thread::sleep(std::time::Duration::from_secs(600));

    Ok(())
}
