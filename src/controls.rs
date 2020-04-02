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


/// Change balance between left/right streams in a stereo setup.
///
/// The balancing is performed by the provided `balance_function` that determines how much of each
/// channels' signal should contribute to a channel's output at any given sample.
pub fn balancer<B>(
    balance_function: B,
    left: Generator,
    right: Generator,
) -> (Generator, Generator)
where
    B: FnMut(Sample, Sample) -> (Sample, Sample) + Send + 'static,
{
    // TODO: use single Arc for whole shared state, instead of 3 Arcs for the three components
    // that are shared?
    let vals_left = Arc::new(Mutex::new((None::<Sample>, None::<Sample>)));
    let vals_right = Arc::clone(&vals_left);

    let generators_left = Arc::new(Mutex::new((left, right)));
    let generators_right = Arc::clone(&generators_left);

    let balance_f_left = Arc::new(Mutex::new(balance_function));
    let balance_f_right = Arc::clone(&balance_f_left);

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
                let ref mut balance_f = &mut *balance_f_left.lock().unwrap();
                let (l, r) = balance_f(left_gen(), right_gen());
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
                let ref mut balance_f = &mut *balance_f_right.lock().unwrap();
                let (l, r) = balance_f(left_gen(), right_gen());
                *vals_unlocked = (Some(l), None);
                r
            },
        }
    });

    (out_left, out_right)
}
