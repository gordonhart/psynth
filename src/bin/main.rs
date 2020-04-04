use anyhow::{anyhow, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use psynth::{
    generators,
    filters,
    consumers,
    controls,
    sampling,
    Consumer,
    FilterComposable,
    Sample,
};
use psynth::music::notes;


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

    /*
    // not source controlled, ripped from freesound.org (awesome website!)
    let kick = "../wavs/371192__karolist__acoustic-kick.wav";
    let hum = "../wavs/17231__meatball4u__hum2.wav";
    let mut consumer = consumers::MonoConsumer::new(channels)
        .bind(controls::join(vec![
            generators::metronome(rate, 120.0, sampling::VecTrack::try_from_wav_file(rate, kick)?)
                .fork(|l, r| controls::join2(
                    controls::GeneratorPot::new(generators::square(rate, 1.0 / 4.0)),
                    l.compose(filters::comb(rate, 0.25, 0.25, filters::CombDirection::FeedBack)),
                    r)),
            generators::sine(rate, controls::StdinPot::default())
                .compose(filters::gain(0.1))
                .compose(filters::reverb(rate, 0.0, 0.0))
                .compose(filters::gain(controls::sine_pot(rate, 1.0 / 3.0, 0.0, 1.0))),
            generators::repeat(sampling::VecTrack::try_from_wav_file(rate, hum)?)
                .compose(filters::gain(0.25)),
            ]));
    */

    let mut consumer = consumers::MonoConsumer::new(channels)
        .bind(generators::sine(rate, controls::StdinPot::new("tone", notes::Tone::FIXED_HZ,
            |line| Ok(notes::Hz::from(notes::Tone::try_from(line)?)))));

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
