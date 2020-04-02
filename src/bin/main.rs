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
    Observer,
};


fn main() -> Result<()> {
    let host = cpal::default_host();
    let output_device = host
        .default_output_device()
        .ok_or_else(|| anyhow!("missing default output device"))?;
    // let config_supported = output_device.default_output_config()?;
    // let config: cpal::StreamConfig = config_supported.into();
    let config = cpal::StreamConfig { channels: 2, sample_rate: cpal::SampleRate(44100) };
    println!("config: {:?}", config);

    let channels = config.channels as usize;
    let rate: u32 = config.sample_rate.0;
    /*
    let left_generator = generators::sine(rate, 200.0)
        .compose(filters::gain(controls::sine_pot(rate, 1.0 / 3.0, 0.0, 1.0)))
        .compose(filters::gain(0.5));
    let right_generator = generators::sine(rate, 250.0)
        .compose(filters::gain(controls::sine_pot(rate, 1.0 / 2.0, 0.0, 1.0)))
        .compose(filters::gain(0.5));

    let mut phaser = generators::sine(rate, 0.9)
        .compose(filters::offset(1.0))
        // .compose(filters::gain(0.5));
        ;
    let (l_new, r_new) = controls::balancer(
        move |l, r| {
            let phase = phaser();
            (l * phase, r * (1.0 - phase).abs())
        },
        left_generator,
        right_generator,
    );

    let generator: psynth::Generator =
        generators::multi(vec![
            generators::microphone(&host, &config)
                .compose(filters::reverb(rate, 0.0, 0.0)),
            generators::sine(rate, 200.0)
                .compose(filters::gain(controls::sine_pot(rate, 1.0 / 3.0, 0.0, 1.0)))
                .compose(filters::gain(0.5)),
            generators::sine(rate, controls::StdinPot::default())
                .compose(filters::gain(controls::sine_pot(rate, 1.0 / 4.0, 0.0, 1.0)))])
        .compose(filters::ramp_up(rate, 0.01))
        .compose(filters::reverb(rate, 0.0, 0.0))
        .compose(filters::gain(0.025))
        ;

    let _observers: Vec<Box<dyn Observer + Send>> = vec![
        // Box::new(std::io::stdout())
    ];
    /*
    let mut consumer = consumers::MonoConsumer::new(channels)
        .bind(generator)
        .bind_observers(observers)
        ;
    */
    let mut consumer = consumers::StereoConsumer::new(channels)
        // .bind(left_generator, right_generator)
        // .bind(l_new, r_new)
        // .bind(generators::metronome(rate, controls::sine_pot(rate, 0.03, 40.0, 200.0)), r_new)
        .bind(
            // generators::metronome(rate, controls::sine_pot(rate, 0.2, 50.0, 200.0),
                // sampling::SampleTrack::from_generator(generators::sine(rate, 440.0), 1000)),
            generators::metronome(rate, 120.0, track),
            r_new)
        ;
    */

    let kick = "../wavs/371192__karolist__acoustic-kick.wav";
    let hum = "../wavs/17231__meatball4u__hum2.wav";
    let gen = controls::join(vec![
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
        ]);
    /*
    let gen = controls::join(vec![
        generators::metronome(rate, 120.0,
            sampling::VecTrack::try_from_wav_file(rate, kick)?)
            .compose(filters::gain(0.25))
            .compose(filters::reverb(rate, 0.0, 0.0)),
        generators::repeat(
            sampling::VecTrack::try_from_wav_file(rate, hum)?)
            .compose(filters::gain(0.1)),
        ]);
    */
    /*
    let mut phaser = generators::square(rate, 0.25)
        .compose(filters::offset(1.0))
        .compose(filters::gain(0.5))
        ;
    let (left_generator, right_generator) = controls::fork(gen);
    let (l_new, r_new) = controls::balancer(
        move |l, r| {
            let phase = phaser();
            (l * phase, r * (1.0 - phase))
        },
        left_generator,
        right_generator,
    );
    */
    let (l_new, r_new) = controls::fork(gen);
    let mut consumer = consumers::StereoConsumer::new(channels)
        .bind(l_new, r_new)
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
