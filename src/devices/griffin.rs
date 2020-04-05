//! Driver for the Griffin PowerMate USB.
//!
//! Product number: `MASM-03169`
//!
//! USB incremental rotary encoder + button combo.

use std::cell::Cell;
use std::fs::File;
use std::sync::mpsc;
use std::thread;
use std::io::Read;

use anyhow::Result;

use crate::Pot;

/*
hacking notes


quick python 'driver':
```
>>> f = open("/dev/input/powermate", "r+b", buffering=0)  # r+ and buffering=0 if we want to write
>>> while True: print(" ".join("{:02X}".format(x) for x in f.read(24)))
```

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
-----> based on this, we can ignore the first 16 timestamp bytes


example events:
    keypress (unreliable):
        down:
            10 03 89 5E 00 00 00 00 08 37 0F 00 00 00 00 00 01 00 00 01 01 00 00 00
        release:
            11 03 89 5E 00 00 00 00 FD 8A 01 00 00 00 00 00 01 00 00 01 00 00 00 00

    rotation:
        clockwise:
            AB 11 89 5E 00 00 00 00 2C 20 0C 00 00 00 00 00 02 00 07 00 01 00 00 00
        counterclockwise:
            AB 11 89 5E 00 00 00 00 74 EC 04 00 00 00 00 00 02 00 07 00 FF FF FF FF


udev rules derived from https://github.com/stefansundin/powermate-linux:

```
# root, group rw access, everybody r access
ACTION=="add", ENV{ID_USB_DRIVER}=="powermate", GROUP="dialout", MODE="0664", SYMLINK+="input/powermate"
```

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


worked on a full Linux machine, did not work out-of-the-box on the TX2
    - was not being properly identified as an input device and mounted in `/dev/input`
    - shows up fine via `lsusb`, exists in `/sys/devices/` and `/dev/bus/usb`
    - kernel logs are fine up until we would expect to see messages from `input` (then followed by
      `usbcore`, `powermate`, `hidraw`, `usbhid` messages)
    - pretty sure that the requisite kernel module (`powermate` and maybe some of its dependencies)
      were not included in the lightweight kernel that came pre-flashed on the board from Nvidia
        - recompiled the kernel (with the `.config` addition: `CONFIG_INPUT_POWERMATE=y` which was
          previously commented out) following the wonderful instructions from jetsonhacks:
          https://www.jetsonhacks.com/2017/03/25/build-kernel-and-modules-nvidia-jetson-tx2/
        - rebooted, and...
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
