use std::thread;
use std::sync::mpsc;

use anyhow::Result;
use byteorder::{BigEndian, ByteOrder};


/// Write the output stream as generated from the `next_sample` function.
///
/// All channels of the output stream are written with the same data.
pub fn write_data<T>(
    output: &mut [T],
    channels: usize,
    next_sample: &mut impl FnMut() -> f32,
)
where
    T: cpal::Sample,
{
    for frame in output.chunks_mut(channels) {
        let value: T = cpal::Sample::from::<f32>(&next_sample());
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}


/// Generate a flat tone of the provided frequency indefinitely.
///
/// Taken almost verbatim from `cpal` examples.
pub fn flat(
    config: &cpal::StreamConfig,
    frequency: f32,
) -> impl FnMut() -> f32 + Send {

    let sample_rate = config.sample_rate.0 as f32;

    // produce a sinusoid of maximum amplitude
    let mut sample_clock = 0f32;
    move || {
        sample_clock = (sample_clock + 1.0) % sample_rate;
        (sample_clock * frequency * 2.0 * std::f32::consts::PI / sample_rate).sin()
    }
}


/// Expose a ZMQ SUB interface to play audio streamed from another process.
///
/// Bytes received are interpreted as big-endian-packed 32-bit floats. All receipt is done on a
/// background thread and stuffed into a channel, allowing arbitrarily large `recv` packet sizes
/// without yielding choppy audio.
pub fn sub_server(
    line: u8,
) -> Result<Box<dyn FnMut() -> f32 + Send>> {

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


/*
// TODO: is there a way to use a `where` clause here to avoid all of this repetition?
pub fn tee(
    generator: impl FnMut(&mut [f32]) + Send + 'static,
    consumer_a: impl FnMut(&mut [f32]) + Send + 'static,
    consumer_b: impl FnMut(&mut [f32]) + Send + 'static,
) -> impl FnMut(&mut [f32]) + Send + 'static {

    const BUFSIZE: usize = 1_000_000;
    let mut buffer = Box::new([0.0f32; BUFSIZE]);

    move |data: &mut [f32]| {
        write_data(data, channels, &mut get
    }
}
*/
