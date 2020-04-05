use anyhow::{anyhow, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

#[allow(unused_imports)]
use psynth::{
    generators,
    filters,
    consumers,
    controls,
    sampling,
    Consumer,
    FilterComposable,
    Sample,
    music::notes,
    devices,
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

    // demonstrate a key reading `true`/`false` values from stdin, with hardcoded attack and
    // sustain functions that ramp by t^2
    let mut consumer = consumers::MonoConsumer::new(channels)
        .bind(psynth::keys::SimpleKey::new(
            generators::sine(rate, notes::Hz::from(notes::Tone::try_from("F#3")?)),
            controls::StdinPot::new("bool", false, |l| Ok(l.parse::<bool>()?)),
            move |i| {
                let i_frac = (i as f32) / (rate as f32);
                if i_frac >= 1.0 { 1.0 } else { i_frac * i_frac }
            },
            move |i| {
                let i_frac = (i as f32) / (rate as f32);
                if i_frac >= 1.0 { 0.0 } else { (1.0 - i_frac) * (1.0 - i_frac) }
            },
            ).into_generator()
                // use PowerMate to control gain
                // start at 0, min 0, max 1, step by 0.01
                .compose(filters::gain(devices::griffin::PowerMateUsbPot::new(0.0, 0.0, 1.0, 0.01)?))
        )
        // .bind_observers(vec![Box::new(std::io::stdout())])
        ;

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
