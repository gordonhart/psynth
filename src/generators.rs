use std::collections::VecDeque;

use anyhow::Result;
use byteorder::{BigEndian, ByteOrder};

use crate::write_data;


pub fn flat(
    config: &cpal::StreamConfig,
    frequency: f32,
) -> impl FnMut(&mut [f32]) + Send + 'static {

    let sample_rate = config.sample_rate.0 as f32;
    println!("sample rate: {}", sample_rate);
    let channels = config.channels as usize;

    // Produce a sinusoid of maximum amplitude.
    let mut sample_clock = 0f32;
    let mut next_value = move || {
        sample_clock = (sample_clock + 1.0) % sample_rate;
        (sample_clock * frequency * 2.0 * 3.141592 / sample_rate).sin()
    };

    move |data: &mut [f32]| write_data(data, channels, &mut next_value)
}


pub fn server(
    config: &cpal::StreamConfig
) -> Result<Box<FnMut(&mut [f32]) + Send + 'static>> {

    let channels = config.channels as usize;

    let ctx = zmq::Context::new();
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
                        eprintln!("WARNING: ignoring trailing {} bytes that do not align", new_len % 4);
                    }
                    if buffer.len() + new_len > buffer.capacity() {
                        buffer.reserve(buffer.len() + new_len - buffer.capacity());
                    }
                    for i in 0 .. new_len / 4 {
                        buffer.push_back(BigEndian::read_f32(&new[i * 4 .. i * 4 + 4]));
                    };
                },
                Err(zmq::Error::EAGAIN) => (),
                Err(e) => panic!("recv panicked: {:?}",  e),
            }
        }
        buffer.pop_front().unwrap_or(0.0)
    };

    Ok(Box::new(move |data: &mut [f32]| write_data(data, channels, &mut get_next_value)))
}
