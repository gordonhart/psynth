//! Driver for the Griffin PowerMate USB.
//!
//! Product number: `MASM-03169`
//!
//! USB incremental rotary encoder + button combo.

use std::cell::Cell;
use std::fs::File;
use std::path::Path;
use std::sync::mpsc;
use std::thread;
use std::io::Read;

use anyhow::Result;

use crate::Pot;

/*
python:
>>> f = open("/dev/input/powermate", "r+b", buffering=0)
>>> while True: print(" ".join("{:02X}".format(x) for x in f.read(24)))

- usually 48-byte chunks are yielded, sometimes 24-byte -- seems safest to read in 24-byte chunks
  and ignore packets that don't look like an expected 24-byte packet


yields input_event from input.h (https://docs.rs/input-linux-sys/0.3.1/input_linux_sys/struct.input_event.html):
pub struct input_event {
    pub time: timeval {
        pub tv_sec: i64,
        pub tv_usev: i64,
    },
    pub type_: u16,
    pub code: u16,
    pub value: i32,
}
== 8 + 8 + 2 + 2 + 4 = 24 ... tada!
so based on this, we can ignore the first 16 timestamp bytes


keypress (unreliable -- haven't fully figured out behavior):
    down:
        68f5885e00000000713b0a00000000000100000101000000
    release:
        85f5885e00000000969d0400000000000100000100000000

rotation:
    clockwise:
        e7f5885e0000000004c90700000000000200070001000000
    counterclockwise:
        f9f5885e000000009b3e04000000000002000700ffffffff


udev rules derived from https://github.com/stefansundin/powermate-linux:

# root, group rw access, everybody r access
ACTION=="add", ENV{ID_USB_DRIVER}=="powermate", GROUP="dialout", MODE="0664", SYMLINK+="input/powermate"

place at /etc/udev/rules.d/60-powermate.rules
add user to dialout: usermod -aG dialout $USER


linux kernel driver: https://elixir.bootlin.com/linux/v5.6.2/source/drivers/input/misc/powermate.c
event codes: https://www.kernel.org/doc/Documentation/input/event-codes.txt

appears to emit EV_SYN between most events (but not always)
emits EV_KEY on keypress and release
    - unreliable -- sometimes behaves as expected, other times not
emits EV_REL on rotation
    - reliable in that rotation appears to always yield an event
    - sometimes (seemingly dependent on messing around with keypresses) the device starts
      repeating each rotation event twice for only one "click" -- can sometimes get out of this
      mode by mashing keypress


-----> choosing to ignore keypresses and only use it as a rotary encoder
*/


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
    const EVENT_LEN: usize = 24;

    pub fn new<P, I>(device_path: P, start: T, min: T, max: T, inc: T) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let mut file = File::open(device_path)?;
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
