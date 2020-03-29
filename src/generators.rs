use std::thread;
use std::sync::mpsc;

use anyhow::Result;
use byteorder::{BigEndian, ByteOrder};

use crate::write_data;


/// Generate a flat tone of the provided frequency indefinitely.
///
/// Taken almost verbatim from `cpal` examples.
pub fn flat(
    config: &cpal::StreamConfig,
    frequency: f32,
) -> impl FnMut(&mut [f32]) + Send + 'static {

    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;

    // Produce a sinusoid of maximum amplitude.
    let mut sample_clock = 0f32;
    let mut next_value = move || {
        sample_clock = (sample_clock + 1.0) % sample_rate;
        (sample_clock * frequency * 2.0 * std::f32::consts::PI / sample_rate).sin()
    };

    move |data: &mut [f32]| write_data(data, channels, &mut next_value)
}


/// Expose a ZMQ SUB interface to play audio streamed from another process.
///
/// Bytes received are interpreted as big-endian-packed 32-bit floats. All receipt is done on a
/// background thread and stuffed into a channel, allowing arbitrarily large `recv` packet sizes
/// without yielding choppy audio.
pub fn sub_server(
    config: &cpal::StreamConfig,
    line: u8,
) -> Result<Box<dyn FnMut(&mut [f32]) + Send + 'static>> {

    let channels = config.channels as usize;

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

    let mut get_next_value = move || receiver.try_recv().unwrap_or(0.0);
    Ok(Box::new(move |data: &mut [f32]| write_data(data, channels, &mut get_next_value)))
}
