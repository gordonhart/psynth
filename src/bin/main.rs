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
    /*
    let f: Sample = 220.0;
    let mut consumer = consumers::MonoConsumer::new(channels)
        .bind(
            controls::join2(controls::StdinPot::default(),
                // even harmonics
                controls::join(vec![
                    generators::sine(rate, f)
                        .compose(filters::ramp_up(rate, 0.05))
                        .compose(filters::gain(0.5)),
                    generators::sine(rate, f * 2.0)
                        .compose(filters::ramp_up(rate, 0.05))
                        .compose(filters::gain(0.05)),
                    generators::sine(rate, f * 4.0)
                        .compose(filters::ramp_up(rate, 0.05))
                        .compose(filters::gain(0.005)),
                ]),
                // odd harmonics
                controls::join(vec![
                    generators::sine(rate, f)
                        .compose(filters::ramp_up(rate, 0.05))
                        .compose(filters::gain(0.5)),
                    generators::sine(rate, f * 1.5)
                        .compose(filters::ramp_up(rate, 0.05))
                        .compose(filters::gain(0.05)),
                    generators::sine(rate, f * 3.0)
                        .compose(filters::ramp_up(rate, 0.05))
                        .compose(filters::gain(0.005)),
                ]),
            )
            // .compose(filters::gain(controls::StdinPot::default()))
            .compose(filters::gain(0.5))
            // .compose(filters::reverb(rate, 0.0, 0.0))
            // .compose(filters::gain(0.25))
            );
    */
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
            ).into_generator())
        ;

    /*
    let mut consumer = consumers::MonoConsumer::new(channels)
        .bind(generators::sine(rate, controls::StdinPot::new("tone", notes::Tone::FIXED_HZ,
            |line| Ok(notes::Hz::from(notes::Tone::try_from(line)?)))));
    */

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
