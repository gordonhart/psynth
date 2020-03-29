use std::thread;
use std::sync::mpsc;

use anyhow::Result;
use byteorder::{BigEndian, ByteOrder};

use crate::Generator;


/// Generate a flat tone of the provided frequency indefinitely.
///
/// Taken almost verbatim from `cpal` examples.
pub fn flat(config: &cpal::StreamConfig, frequency: f32) -> Generator {

    let sample_rate = config.sample_rate.0 as f32;

    // produce a sinusoid of maximum amplitude
    let mut sample_clock = 0f32;
    Box::new(move || {
        sample_clock = (sample_clock + 1.0) % sample_rate;
        (sample_clock * frequency * 2.0 * std::f32::consts::PI / sample_rate).sin()
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
