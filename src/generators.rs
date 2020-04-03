use std::f32::consts::PI;
use std::sync::mpsc;
use std::thread;

use anyhow::Result;
use byteorder::{BigEndian, ByteOrder};
use cpal::traits::{HostTrait, DeviceTrait, StreamTrait};
use ringbuf::RingBuffer;

use crate::{Generator, Pot};
use crate::sampling::SampleTrack;


/// Generate a sine wave of the provided frequency indefinitely and with maximum amplitude (-1, 1).
pub fn sine<P>(sample_rate: u32, frequency: P) -> Generator
where
    P: Pot<f32> + 'static,
{
    let rate = sample_rate as f32;
    let mut t = 0u64;
    Box::new(move || {
        t = t + 1;
        ((t as f32) * frequency.read() * 2.0 * PI / rate).sin()
    })
}


/// Generate a square wave tone of the provided frequncy indefinitely.
pub fn square(sample_rate: u32, frequency: f32) -> Generator {
    let mut gen = sine(sample_rate, frequency);
    Box::new(move || {
        let value = gen();
        if value > 0.0 { 1.0 } else { -1.0 }
    })
}


/// Generate a sawtooth wave of the provided frequncy indefinitely.
pub fn sawtooth(sample_rate: u32, frequency: f32) -> Generator {
    let rate = sample_rate as f32;
    let mut sample_clock = 0f32;
    Box::new(move || {
        sample_clock = (sample_clock + 1.0) % rate;
        let val = frequency * (sample_clock / rate);
        val - val.floor()
    })
}


/// Play back the provided `track` once per beat at the requested `bpm`.
///
/// For good results, the duration of the `track` should be less than the requested time between
/// beats (e.g. a drum kick, not a whole song).
pub fn metronome<P, T>(sample_rate: u32, bpm: P, mut track: T) -> Generator
where
    P: Pot<f32> + 'static,
    T: SampleTrack + Send + 'static,
{
    let rate = sample_rate as f32;
    let mut sample_clock = 0f32;

    Box::new(move || {
        let n_steps_between_ticks = 60.0 * rate / bpm.read();
        sample_clock += 1.0;
        if sample_clock > n_steps_between_ticks {
            sample_clock = 0.0;
            track.reset();
        }
        track.next().unwrap_or_else(|| 0.0)
    })
}


/// Never creates any sound.
pub fn silence() -> Generator {
    Box::new(move || 0.0)
}


/// Repeatedly loop through the provided track indefinitely.
pub fn repeat<T>(mut track: T) -> Generator
where
    T: SampleTrack + Send + 'static,
{
    Box::new(move || {
        track.next().unwrap_or_else(|| {
            track.reset();
            track.next().unwrap_or_else(|| 0.0)
        })
    })
}


/// Spawn the default system input device as a `Generator`.
pub fn microphone(
    host: &cpal::Host,
    output_config: &cpal::StreamConfig,
) -> Generator {

    let input_device = host
        .default_input_device()
        .expect("failed to get default input device");

    let input_config: cpal::StreamConfig = input_device
        .default_input_config()
        .expect("failed to get default input config")
        .into();

    let mut sample_factor: u32 = 1;
    let (isr, osr) = (input_config.sample_rate.0, output_config.sample_rate.0);
    if isr < osr {
        if osr % isr != 0 {
            unimplemented!(
                "TODO: handle output sample rate % input sample rate != 0\ninput: {:?}\noutput: {:?}",
                isr, osr
            );
        }
        sample_factor = osr / isr;
    } else if input_config.sample_rate.0 > output_config.sample_rate.0 {
        unimplemented!(
            "TODO: handle input sample rate > output sample rate\ninput: {:?}\noutput: {:?}",
            isr, osr
        );
    }

    const BUFSIZE: usize = 2048;
    let ring = RingBuffer::new(2 * BUFSIZE);
    let (mut producer, mut consumer) = ring.split();

    thread::spawn(move || {
        let input_data_fn = move |data: &[f32]| {
            let mut output_fell_behind = false;
            for &sample in data {
                for _ in 0 .. sample_factor {
                    if producer.push(sample).is_err() {
                        output_fell_behind = true;
                    }
                }
            }
            if output_fell_behind {
                eprintln!("output stream fell behind: try increasing latency");
            }
        };

        println!("attempting to build input stream with f32 samples with config: {:?}", input_config);
        let input_stream = input_device.build_input_stream(
            &input_config,
            input_data_fn,
            move |err| panic!("input stream error: {:?}", err),
        ).expect("failed to build input stream");
        input_stream.play().expect("failed to start mic");
        println!("input stream playing");

        // FIXME: ugly hack to keep this thread alive (and thus the stream running)
        std::thread::sleep(std::time::Duration::from_secs(u64::max_value()));
    });

    Box::new(move || {
        consumer.pop().unwrap_or_else(|| {
            // eprintln!("input stream fell behind");
            0.0
        })
    })
}


/// Expose a ZMQ SUB interface to play audio streamed from another process.
///
/// Bytes received are interpreted as big-endian-packed 32-bit floats. All receipt is done on a
/// background thread and stuffed into a channel, allowing arbitrarily large `recv` packet sizes
/// without yielding choppy audio.
pub fn sub_server(line: u8) -> Result<Generator> {

    let ctx = zmq::Context::new();
    let socket = ctx.socket(zmq::SUB)?;
    socket.set_subscribe(&[])?;
    let endpoint = format!("ipc:///tmp/.psynth.{}", line);
    println!("subcribing on '{}'", endpoint);
    socket.connect(endpoint.as_str())?;

    let (sender, receiver) = mpsc::channel();

    thread::spawn(move || loop {
        match socket.recv_bytes(0) {
            Ok(new) => {
                let new_len = new.len();
                if new_len % 4 != 0 {
                    eprintln!("WARN: ignoring trailing {} bytes that do not align", new_len % 4);
                }
                for i in 0 .. new_len / 4 {
                    let new_value = BigEndian::read_f32(&new[i * 4 .. i * 4 + 4]);
                    sender.send(new_value).expect("channel send failed");
                }
            },
            Err(e) => panic!("recv panicked: {:?}",  e),
        }
    });
    
    Ok(Box::new(move || receiver.try_recv().unwrap_or(0.0)))
}
