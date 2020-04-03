use std::cell::{RefCell, Cell};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::{generators, filters, Pot, Generator, FilterComposable, Sample};


/// Allow for the usage of raw floats as `f32` potentiometers when control over the value is not
/// necessary.
impl Pot<f32> for f32 {
    fn read(&self) -> f32 {
        *self
    }
}


/// Modulate the value returned via the held `Generator`.
///
/// Useful for composing `Generators` as `Pot` inputs to other `Generator`s or `Filter`s.
pub struct GeneratorPot {
    gen: RefCell<Generator>,
}

impl GeneratorPot {
    pub fn new(generator: Generator) -> Self {
        Self {
            gen: RefCell::new(generator),
        }
    }
}

impl Pot<f32> for GeneratorPot {
    fn read(&self) -> f32 {
        (&mut *self.gen.borrow_mut())()
    }
}


/// Sinusoidally osciallating potentiometer.
///
/// Crank back and forth at the provided `frequency`, oscillating between the provided `low` and
/// `high` values.
pub fn sine_pot<P>(sample_rate: u32, frequency: P, low: f32, high: f32) -> GeneratorPot
where
    P: Pot<f32> + 'static
{
    GeneratorPot::new(
        generators::sine(sample_rate, frequency)
            .compose(filters::offset(1.0))
            .compose(filters::gain((high - low) / 2.0))
            .compose(filters::offset(low)))
}


/// Interactively read values from stdin via `readline`.
pub struct StdinPot {
    cur: Cell<f32>,
    receiver: std::sync::mpsc::Receiver<f32>,
}

impl Default for StdinPot {
    fn default() -> Self {
        Self::new("StdinPot")
    }
}

impl StdinPot {
    /// Create a new `StdinPot`, spawning a stdin reader thread.
    fn new(name: &str) -> Self {
        let prompt = format!("{}> ", name);
        let mut reader = rustyline::Editor::<()>::new();
        let (sender, receiver) = std::sync::mpsc::channel();
        thread::spawn(move || loop {
            match reader.readline(prompt.as_str()) {
                Ok(l) if l == "q" => {
                    println!("exit requested, exiting gracefully...");
                    std::process::exit(0);
                },
                Ok(l) => {
                    match l.parse::<f32>() {
                        Ok(val) => {
                            sender.send(val).unwrap();  // thread panic if channel is closed
                            reader.add_history_entry(l.as_str());
                        },
                        Err(e) => eprintln!("'{}' is not a float, try again (reason: {:?})", l, e),
                    }
                },
                Err(e) => {
                    eprintln!("readline error: {:?}", e);
                    std::process::exit(1);
                },
            }
        });
        Self {
            cur: Cell::new(0.0),
            receiver: receiver,
        }
    }
}

impl Pot<f32> for StdinPot {
    fn read(&self) -> f32 {
        match self.receiver.try_recv() {
            Ok(val) => {
                self.cur.set(val);
                val
            },
            Err(_) => self.cur.get(),
        }
    }
}


/// Mux together two left/right streams in a stereo setup using a custom mux function.
///
/// The muxing is performed by the provided `mux_function` that determines how much of each
/// channels' signal should contribute to a channel's output at any given sample.
///
/// The yielded `Generator`s are entangled in that calling one also calls the other. This is
/// important to take note of for `Generator` implementations that keep some sort of internal state.
pub fn mux2<F>(
    mux_function: F,
    left: Generator,
    right: Generator,
) -> (Generator, Generator)
where
    F: FnMut(Sample, Sample) -> (Sample, Sample) + Send + 'static,
{
    // TODO: use single Arc for whole shared state, instead of 3 Arcs for the three components
    // that are shared?
    let vals_left = Arc::new(Mutex::new((None::<Sample>, None::<Sample>)));
    let vals_right = Arc::clone(&vals_left);

    let generators_left = Arc::new(Mutex::new((left, right)));
    let generators_right = Arc::clone(&generators_left);

    let mux_f_left = Arc::new(Mutex::new(mux_function));
    let mux_f_right = Arc::clone(&mux_f_left);

    let out_left: Generator = Box::new(move || {
        let mut vals_unlocked = vals_left.lock().unwrap();
        match *vals_unlocked {
            (Some(_), Some(_)) => unreachable!("neither value collected -- should never occur"),
            (Some(l), None) => {
                *vals_unlocked = (None, None);
                l
            },
            (None, _) => {
                let (ref mut left_gen, ref mut right_gen) = &mut *generators_left.lock().unwrap();
                let ref mut mux_f = &mut *mux_f_left.lock().unwrap();
                let (l, r) = mux_f(left_gen(), right_gen());
                *vals_unlocked = (None, Some(r));
                l
            },
        }
    });

    // this would get tedious for more than two channels -- is there a general-form solution for
    // this multi-stream muxing problem?
    let out_right: Generator = Box::new(move || {
        let mut vals_unlocked = vals_right.lock().unwrap();
        match *vals_unlocked {
            (Some(_), Some(_)) => unreachable!("neither value collected -- should never occur"),
            (None, Some(r)) => {
                *vals_unlocked = (None, None);
                r
            },
            (_, None) => {
                let (ref mut left_gen, ref mut right_gen) = &mut *generators_right.lock().unwrap();
                let ref mut mux_f = &mut *mux_f_right.lock().unwrap();
                let (l, r) = mux_f(left_gen(), right_gen());
                *vals_unlocked = (Some(l), None);
                r
            },
        }
    });

    (out_left, out_right)
}


/// Fork the provided `Generator` into two entangled `Generator`s that will yield the same value
/// on each at any given instant.
pub fn fork(generator: Generator) -> (Generator, Generator) {
    mux2(move |l, _| (l, l), generator, generators::silence())
}


/// Join the provided `Generator` streams into a single `Generator`.
///
/// Allows composition of multiple input sources. Serves a similar purpose for `Generator`s as
/// `filters::parallel` serves for `Filter`s.
pub fn join(mut generators: Vec<Generator>) -> Generator {
    Box::new(move || {
        let mut out = 0f32;
        for generator in generators.iter_mut() {
            out += generator();
        }
        out
    })
}


/// Join the two `Generator`s into a single `Generator`.
///
/// The `bias` potentiometer determines how much of each signal contributes to the end result.
/// A value of `≤-1` is 100% `left`, `≥1` is 100% `right`, and 0 is an even 50%/50% split.
pub fn join2<P>(bias: P, mut left: Generator, mut right: Generator) -> Generator
where
    P: Pot<f32> + 'static,
{
    let mut clipper = filters::clip(-1.0, 1.0);
    Box::new(move || {
        let l_val = left();
        let r_val = right();
        let b_val = clipper(bias.read()) + 1.0;
        ((2.0 - b_val) * 0.5 * l_val) + (b_val * 0.5 * r_val)
    })
}


pub fn balance<P>(
    _balance_pot: P,
    _left: Generator,
    _right: Generator,
) -> (Generator, Generator)
where
    P: FnMut(Sample, Sample) -> (Sample, Sample) + Send + 'static,
{
    unimplemented!()
}


#[cfg(feature = "hardware")]
pub mod hardware {
    use super::*;

    use std::sync::mpsc;
    use std::thread;

    use anyhow::Result;
    use embedded_hal::blocking::i2c::Read;
    use linux_embedded_hal::I2cdev;

    pub struct BlockingI2cPot<T> {
        address: u8,
        device: RefCell<I2cdev>,
        buffer: RefCell<Vec<u8>>,
        converter: Box<dyn Fn(&[u8]) -> T + Send>,
    }

    impl<T> BlockingI2cPot<T> {
        pub fn new<F>(
            bus: u8,
            address: u8,
            message_size: usize, // number of bytes that comprise a single reading
            converter: F,
        ) -> Result<BlockingI2cPot<T>>
        where
            F: Fn(&[u8]) -> T + Send + 'static,
        {
            let mut device = I2cdev::new(format!("/dev/i2c-{}", bus))?;
            device.set_slave_address(address as u16)?;
            Ok(BlockingI2cPot {
                address: address,
                device: RefCell::new(device),
                buffer: RefCell::new(vec![0; message_size]),
                converter: Box::new(converter),
            })
        }
    }

    impl<T> Pot<T> for BlockingI2cPot<T> {
        fn read(&self) -> T {
            let mut device = self.device.borrow_mut();
            let mut buffer = self.buffer.borrow_mut();
            // TODO: not panic
            device.read(self.address, buffer.as_mut_slice()).expect("i2c error");
            println!("{:?}", buffer);
            (self.converter)(buffer.as_slice())
        }
    }

    pub struct ThreadedI2cPot<T: Sized + Send + Copy> {
        receiver: mpsc::Receiver<T>,
        latest: Cell<T>,
    }

    impl<T> ThreadedI2cPot<T>
    where
        T: Sized + Send + Copy + 'static, // static required for thread::spawn
    {
        pub fn new<F>(
            bus: u8,
            address: u8,
            message_size: usize,
            converter: F,
        ) -> Result<ThreadedI2cPot<T>>
        where
            F: Fn(&[u8]) -> T + Send + 'static,
        {
            let (sender, receiver) = mpsc::channel();

            let blocking_pot = BlockingI2cPot::new(bus, address, message_size, converter)?;
            thread::spawn(move || loop {
                let new_val = blocking_pot.read();
                sender.send(new_val).expect("channel closed");
            });

            let first_val = receiver.recv()?;
            Ok(ThreadedI2cPot {
                receiver: receiver,
                latest: Cell::new(first_val),
            })
        }
    }

    impl<T> Pot<T> for ThreadedI2cPot<T>
    where
        T: Sized + Send + Copy,
    {
        fn read(&self) -> T {
            match self.receiver.try_recv() {
                Ok(val) => {
                    self.latest.set(val);
                    val
                },
                Err(_) => self.latest.get(),
            }
        }
    }
}
