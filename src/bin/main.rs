use anyhow::{anyhow, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

#[allow(unused_imports)]
use psynth::{
    generators,
    filters,
    consumers,
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

    /*
    // demonstrate a key reading `true`/`false` values from stdin, with hardcoded attack and
    // sustain functions that ramp by t^2
    let mut consumer = consumers::MonoConsumer::new(channels)
        .bind(psynth::keys::SimpleKey::new(
            // generators::sine(rate, notes::Hz::from(notes::Tone::try_from("F#3")?)),
            generators::white(),
            controls::StdinPot::new("bool", false, |l| Ok(l.parse::<bool>()?)),
            move |i| {
                let i_frac = (i as f32) / (rate as f32);
                if i_frac >= 0.05 { 1.0 } else { (20.0 * i_frac) * (20.0 * i_frac) }
            },
            move |i| {
                let i_frac = (i as f32) / (rate as f32);
                if i_frac >= 1.0 { 0.0 } else { (1.0 - i_frac) * (1.0 - i_frac) }
            },
            ).into_generator()
                // use PowerMate to control gain
                // start at 0, min 0, max 1, step by 0.01
                // .compose(filters::gain(devices::griffin::PowerMateUsbPot::new(0.0, 0.0, 1.0, 0.01)?))
                .compose(filters::gain(0.25))
        )
        // .bind_observers(vec![Box::new(std::io::stdout())])
        ;
    */

    let mut consumer = consumers::MonoConsumer::new(channels)
        .bind(
            // generators::sine(rate, controls::StdinPot::default())
            /*
            generators::sine(rate, 880.0)
                // .compose(filters::single_pole_low_pass(controls::StdinPot::default()))
                // .compose(filters::single_pole_high_pass(controls::StdinPot::default()))
                // .compose(filters::four_stage_low_pass(controls::StdinPot::default()))
                // .compose(filters::band_pass(rate, 440.0, 10.0))
                // .compose(filters::band_pass(rate, controls::StdinPot::new("(0,inf)", 100.0, |line| Ok(line.parse::<f64>()?)), 100.0))
                .compose(filters::band_pass(rate, controls::sine_pot(rate, 1.0, 440.0, 880.0), 50.0))
                */
            /*
            controls::join2(
                0.0,
                generators::sine(rate, 880.0),
                generators::sine(rate, 440.0),
            )
            */
            control::flow::join(vec![
                generators::sine(rate, 220.0),
                generators::sine(rate, 440.0),
                generators::sine(rate, 660.0),
                generators::sine(rate, 880.0),
                generators::sine(rate, 1100.0),
            ])
                .compose(filters::band_pass(rate, control::pot::sine_pot(rate, 1.0, 440.0, 880.0), 80.0))
                .compose(filters::gain(0.1))
        )
        // .bind_observers(vec![Box::new(std::io::stdout())])
        ;

    /*
    // let f_hz = notes::Tone::FIXED_HZ / 2.0;
    let harmonic_gain = 0.05;
    let f_hz = std::sync::Arc::new(std::sync::Mutex::new(controls::StdinPot::default()));
    let f_hz_2 = f_hz.clone();
    let f_hz_3 = f_hz.clone();
    let f_hz_4 = f_hz.clone();
    let mut consumer = consumers::MonoConsumer::new(channels)
        .bind(controls::join2(
            0.0,
            // fundamental
            generators::sine(rate, f_hz.clone()),
            // harmonics
            controls::join(vec![
                generators::sine(rate, move || 2.0 * f_hz_2.read()).compose(filters::gain(harmonic_gain)),
                generators::sine(rate, move || 3.0 * f_hz_3.read()).compose(filters::gain(harmonic_gain)),
                generators::sine(rate, move || 4.0 * f_hz_4.read()).compose(filters::gain(harmonic_gain)),

                // generators::sine(rate, 2.0 * f_hz).compose(filters::gain(harmonic_gain)),
                // generators::sine(rate, 3.0 * f_hz).compose(filters::gain(harmonic_gain)),
                // generators::sine(rate, 4.0 * f_hz).compose(filters::gain(harmonic_gain)),
                /*
                generators::sine(rate, 5.0 * f_hz).compose(filters::gain(harmonic_gain)),
                */
                // generators::sine(rate, 6.0 * f_hz).compose(filters::gain(harmonic_gain)),
                /*
                generators::sine(rate, 7.0 * f_hz).compose(filters::gain(harmonic_gain)),
                generators::sine(rate, 8.0 * f_hz).compose(filters::gain(harmonic_gain)),
                generators::sine(rate, 9.0 * f_hz).compose(filters::gain(harmonic_gain)),
                generators::sine(rate, 10.0 * f_hz).compose(filters::gain(harmonic_gain)),
                */
            ])
        ).compose(filters::gain(0.5)));
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
