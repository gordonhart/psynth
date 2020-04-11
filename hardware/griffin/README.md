# Griffin PowerMate USB

## Hacking Notes

- Quick python 'driver':

    ```python
    >>> f = open("/dev/input/powermate", "r+b", buffering=0)  # r+ and buffering=0 if we want to write
    >>> while True: print(" ".join("{:02X}".format(x) for x in f.read(24)))
    ```

- Yields `input_event` from [`input.h`](https://docs.rs/input-linux-sys/0.3.1/input_linux_sys/struct.input_event.html):

    ```rust
    pub struct input_event {
        pub time: timeval {
            pub tv_sec: i64,
            pub tv_usev: i64,
        },
        pub type_: u16,
        pub code: u16,
        pub value: i32,
    }
    ```

    Based on this, we can ignore the first 16 timestamp bytes.

- Example events:

    ```
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
    ```

- `udev` rules derived from https://github.com/stefansundin/powermate-linux:

    ```
    # root, group rw access, everybody r access
    ACTION=="add", ENV{ID_USB_DRIVER}=="powermate", GROUP="dialout", MODE="0664", SYMLINK+="input/powermate"
    ```

    - Place at `/etc/udev/rules.d/60-powermate.rules`
    - Add user to `dialout` group: `usermod -aG dialout $USER`


- Linux kernel driver: https://elixir.bootlin.com/linux/v5.6.2/source/drivers/input/misc/powermate.c

- Event codes: https://www.kernel.org/doc/Documentation/input/event-codes.txt
    - Appears to emit `EV_SYN` between most events (but not always)
    - Emits `EV_KEY` on keypress and release
        - unreliable -- sometimes behaves as expected, other times not
    - Emits `EV_REL` on rotation
        - reliable in that rotation appears to always yield an event
        - sometimes (seemingly dependent on messing around with keypresses) the device starts
          repeating each rotation event twice for only one "click" -- can sometimes get out of this
          mode by mashing keypress
    - Choosing to ignore keypresses and only use it as a rotary encoder


- Worked on a full Linux machine, did not work out-of-the-box on the TX2
    - Was not being properly identified as an input device and mounted in `/dev/input`
    - Shows up fine via `lsusb`, exists in `/sys/devices/` and `/dev/bus/usb`
    - Kernel logs are fine up until we would expect to see messages from `input` (then followed by
      `usbcore`, `powermate`, `hidraw`, `usbhid` messages)
    - Pretty sure that the requisite kernel module (`powermate` and maybe some of its dependencies)
      were not included in the lightweight kernel that came pre-flashed on the board from Nvidia
        - Recompiled the kernel (with the `.config` addition: `CONFIG_INPUT_POWERMATE=y` which was
          previously commented out) following the
          [wonderful instructions from jetsonhacks](https://www.jetsonhacks.com/2017/03/25/build-kernel-and-modules-nvidia-jetson-tx2)
        - Rebooted, and... success! good to have run through these paces now -- will need to do
          something similar to get other hardware running whose drivers are not included in the
          default install
