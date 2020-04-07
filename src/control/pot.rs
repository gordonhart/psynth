use std::cell::{RefCell, Cell};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

use anyhow::Result;

use crate::{generator, filter, Pot, Generator, FilterComposable};


/// Allow for the usage of raw floats as `f32` potentiometers when control over the value is not
/// necessary.
impl Pot<f32> for f32 {
    fn read(&self) -> f32 {
        *self
    }
}


impl Pot<f64> for f64 {
    fn read(&self) -> f64 {
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


// TODO: less manually-intetnsive way to allow easy f32/f64 use in different locations?
impl Pot<f64> for GeneratorPot {
    fn read(&self) -> f64 {
        let r: f32 = self.read();
        r as f64
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
        generator::sine(sample_rate, frequency)
            .compose(filter::offset(1.0))
            .compose(filter::gain((high - low) / 2.0))
            .compose(filter::offset(low)))
}


/// Interactively read values from stdin via `readline`.
pub struct StdinPot<T> {
    cur: Cell<T>,
    receiver: mpsc::Receiver<T>,
}


impl Default for StdinPot<f32> {
    fn default() -> Self {
        Self::new("f32pot", 0.0, |line| Ok(line.parse::<f32>()?))
    }
}


impl Default for StdinPot<f64> {
    fn default() -> Self {
        Self::new("f64pot", 0.0, |line| Ok(line.parse::<f64>()?))
    }
}


impl<T> StdinPot<T>
where
    T: Send + 'static,
{
    /// Create a new `StdinPot`, spawning a stdin reader thread.
    pub fn new<F>(name: &str, default: T, converter: F) -> Self
    where
        F: Fn(&str) -> Result<T> + Send + 'static,
    {
        let prompt = format!("{}> ", name);
        let mut reader = rustyline::Editor::<()>::new();
        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || loop {
            match reader.readline(prompt.as_str()) {
                Ok(l) if l == "q" => {
                    println!("exit requested, exiting gracefully...");
                    std::process::exit(0);
                },
                Ok(l) => {
                    match converter(l.as_str()) {
                        Ok(val) => {
                            sender.send(val).unwrap();  // thread panic if channel is closed
                            reader.add_history_entry(l.as_str());
                        },
                        Err(e) => eprintln!("unable to parse '{}', try again (reason: {:?})", l, e),
                    }
                },
                Err(e) => {
                    eprintln!("readline error: {:?}", e);
                    std::process::exit(1);
                },
            }
        });
        Self {
            cur: Cell::new(default),
            receiver: receiver,
        }
    }
}


impl<T> Pot<T> for StdinPot<T>
where
    T: Send + Copy + 'static,
{
    fn read(&self) -> T {
        match self.receiver.try_recv() {
            Ok(val) => {
                self.cur.set(val);
                val
            },
            Err(_) => self.cur.get(),
        }
    }
}


/// Enables sharing of a `Pot` impl in multiple places.
impl<T, P> Pot<T> for Arc<Mutex<P>>
where
    T: Send + Copy + 'static,
    P: Pot<T> + 'static,
{
    fn read(&self) -> T {
        let inner = self.lock().unwrap();
        (*inner).read()
    }
}


impl<T, F> Pot<T> for F
where
    T: Send + Copy + 'static,
    F: Fn() -> T + Send + 'static,
{
    fn read(&self) -> T {
        (self)()
    }
}
