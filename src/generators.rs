use std::collections::VecDeque;
use std::thread;
use std::time::Duration;
use std::sync::mpsc;

use anyhow::Result;
use byteorder::{BigEndian, ByteOrder};
use zmq::{Context, Socket, PollItem};

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
/// Bytes received are interpreted as big-endian-packed 32-bit floats.
pub fn sub_server_single(
    config: &cpal::StreamConfig,
) -> Result<Box<dyn FnMut(&mut [f32]) + Send + 'static>> {

    let channels = config.channels as usize;

    let ctx = Context::new();
    let socket = ctx.socket(zmq::SUB)?;
    socket.set_subscribe(&[])?;
    socket.connect("ipc:///tmp/.psynth.0")?;

    // NOTE: when large packets (> ~100KB) are received, this function takes too long to produce
    // the next note smoothly
    const PADDING: usize = 1024;
    let mut buffer: VecDeque<f32> = VecDeque::new();

    let mut get_next_value = move || {
        if buffer.len() < PADDING {
            match socket.recv_bytes(zmq::DONTWAIT) {
                Ok(new) => {
                    let new_len = new.len();
                    println!("received {} bytes", new_len);
                    if new_len % 4 != 0 {
                        eprintln!(
                            "WARNING: ignoring trailing {} bytes that do not align", new_len % 4
                        );
                    }
                    if buffer.len() + new_len > buffer.capacity() {
                        buffer.reserve(buffer.len() + new_len - buffer.capacity());
                    }
                    for i in 0 .. new_len / 4 {
                        buffer.push_back(BigEndian::read_f32(&new[i * 4 .. i * 4 + 4]));
                    }
                },

                // nothing to read and didn't hold due to DONTWAIT flag -- do nothing
                Err(zmq::Error::EAGAIN) => (),
                Err(e) => panic!("recv panicked: {:?}",  e),
            }
        }
        buffer.pop_front().unwrap_or(0.0)
    };

    Ok(Box::new(move |data: &mut [f32]| write_data(data, channels, &mut get_next_value)))
}


// TODO: connect to multiple endpoints to allow superposition of audio from multiple sources
pub fn sub_server_multi(
    config: &cpal::StreamConfig,
    n_peers: usize,
) -> Result<Box<dyn FnMut(&mut [f32]) + Send + 'static>> {

    let channels = config.channels as usize;

    let ctx = Context::new();

    let mut sockets: Vec<Socket> = Vec::new();
    for i in 0 .. n_peers {
        let socket = ctx.socket(zmq::SUB)?;
        socket.set_subscribe(&[])?;
        let endpoint = format!("ipc:///tmp/.psynth.{}", i);
        socket.connect(endpoint.as_str())?;
        sockets.push(socket);
    }

    // NOTE: when large packets (> ~100KB) are received, this function takes too long to produce
    // the next note smoothly
    const PADDING: usize = 1024;
    let mut buffer: VecDeque<f32> = VecDeque::new();

    let mut get_next_value = move || {
        // TODO: PollItem is !Send + !Sync, how to access within this closure without recreating
        // every time this is called?
        let mut poll_items = sockets
            .iter()
            .map(|socket| socket.as_poll_item(zmq::PollEvents::POLLIN))
            .collect::<Vec<PollItem>>();
        if buffer.len() < PADDING {
            match zmq::poll(&mut poll_items[..], 0) {
                Ok(0) => (),
                Ok(n) => {
                    for (socket, poller) in sockets.iter().zip(poll_items.iter()) {
                        if !poller.is_readable() {
                            continue;
                        }
                        let new = socket.recv_bytes(zmq::DONTWAIT).expect("missing message?");
                        for i in 0 .. new.len() / 4 {
                            buffer.push_back(BigEndian::read_f32(&new[i * 4 .. i * 4 + 4]));
                        }
                    }
                },

                Err(e) => panic!("poll panicked: {:?}",  e),
            }
        }
        buffer.pop_front().unwrap_or(0.0)
    };

    Ok(Box::new(move |data: &mut [f32]| write_data(data, channels, &mut get_next_value)))
}
