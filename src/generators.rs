use anyhow::Result;
use byteorder::{BigEndian, ReadBytesExt};

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


pub fn server(config: &cpal::StreamConfig) -> Result<Box<FnMut(&mut [f32]) + Send + 'static>> {
    let channels = config.channels as usize;

    let ctx = zmq::Context::new();
    let socket = ctx.socket(zmq::SUB)?;
    socket.set_subscribe(&[])?;
    socket.connect("ipc:///tmp/.psynth.0")?;

    const MAXSIZE: usize = 8192;
    const PADDING: usize = 1024;
    let mut buffer: Vec<u8> = Vec::new();

    let mut get_next_value = move || {
        if buffer.len() < PADDING {
            // let new = match socket.recv_bytes(zmq::DONTWAIT) {
            let new = match socket.recv_bytes(0) {
                Ok(v) => v,
                Err(zmq::Error::EAGAIN) => Vec::new(),
                Err(e) => panic!("recv panicked: {:?}",  e),
            };
            if new.len() > 0 {
                println!("read {}", new.len());
                buffer.extend_from_slice(&new[..]);
            }
        }
        if buffer.len() > 3 {
            // let val = f32::from_be_bytes(&buffer[ptr..ptr+4]);
            let bytes = buffer.drain(..4).collect::<Vec<u8>>();
            // println!("bytes: {:?}", bytes);
            let val = (&bytes[..]).read_f32::<BigEndian>().unwrap();
            // println!("sending: {}", val);
            val
        } else {
            0.0
        }
    };

    Ok(Box::new(move |data: &mut [f32]| write_data(data, channels, &mut get_next_value)))
}
