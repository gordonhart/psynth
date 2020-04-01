use std::cell::{RefCell, Cell};
use std::thread;

use crate::{generators, filters, Pot, Generator, FilterComposable};


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
