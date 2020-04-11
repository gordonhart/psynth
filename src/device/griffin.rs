//! Driver for the Griffin PowerMate USB.
//!
//! Product number: `MASM-03169`
//!
//! USB incremental rotary encoder + button combo.
//!
//! See `hardware/griffin/README.md` for hacking notes to get this device working on a normal Linux
//! and Tegra Linux (L4T) machine.

use std::cell::Cell;
use std::fs::File;
use std::sync::mpsc;
use std::thread;
use std::io::Read;

use anyhow::Result;

use crate::Pot;


pub struct PowerMateUsbPot<T> {
    receiver: mpsc::Receiver<T>,
    latest: Cell<T>,
    min: T,
    max: T,
}


impl<T> PowerMateUsbPot<T>
where
    T: std::ops::Neg<Output = T> + Copy + Send + 'static,
{
    /// Length of events yielded by the device.
    ///
    /// The actual `input_event` struct is described in the kernel's `input.h`.
    const EVENT_LEN: usize = 24;

    /// Hardcoded path where the device interface will be located if the above `udev` rules are
    /// properly applied.
    const DEVICE_PATH: &'static str = "/dev/input/powermate";

    pub fn new(start: T, min: T, max: T, inc: T) -> Result<Self> {
        let mut file = File::open(Self::DEVICE_PATH)?;
        let mut buffer = vec![0u8; Self::EVENT_LEN];
        let (sender, receiver) = mpsc::channel();

        thread::spawn(move || loop {
            file.read(buffer.as_mut_slice()).expect("PowerMate device disappeared");
            // TODO: is it worth the extra baggage to integrate software that understands input.h
            // events a little better than this?
            match (buffer[16], buffer[20]) {
                (0x02, 0xFF) => sender.send(-inc).expect("PowerMateUsb channel closed"),
                (0x02, 0x01) => sender.send(inc).expect("PowerMateUsb channel closed"),
                (0x02, _) => eprintln!("unexpected message received"),
                _ => (),
            };
        });

        Ok(Self {
            receiver,
            latest: Cell::new(start),
            min,
            max,
        })
    }
}


impl Pot<f32> for PowerMateUsbPot<f32> {
    fn read(&self) -> f32 {
        let latest = self.latest.get();
        match self.receiver.try_recv() {
            Ok(inc_val) => {
                let new = (latest + inc_val).max(self.min).min(self.max);
                self.latest.set(new);
                new
            },
            Err(_) => latest,
        }
    }
}
