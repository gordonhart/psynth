use std::cell::{RefCell, Cell};
use std::thread;

use crate::{generators, Pot, Generator};


impl Pot<f32> for f32 {
    fn read(&self) -> f32 {
        *self
    }
}


pub struct TimedSawtoothPot {
    pub low: f32,
    pub high: f32,
    pub period: f32,
}


impl Default for TimedSawtoothPot {
    fn default() -> Self {
        Self {
            low: 0.0,
            high: 1.0,
            period: 1.0,
        }
    }
}


impl Pot<f32> for TimedSawtoothPot {
    fn read(&self) -> f32 {
        /*
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .expect("time moved backwards")
            .subsec_nanos() as f32;
        ((self.high - self.low) * (ts / 1000000000.0)) + self.low
        */
        if std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .expect("time moved backwards")
            .as_secs() % 2 == 0 {
            self.low
        } else {
            self.high
        }
    }
}


pub struct SinePot {
    gen: RefCell<Generator>,
    low: f32,
    high: f32,
}


impl SinePot {
    pub fn new<P>(config: &cpal::StreamConfig, frequency: P, low: f32, high: f32) -> Self
    where
        P: Pot<f32> + 'static
    {
        Self {
            gen: RefCell::new(generators::sine(config, frequency)),
            low: low,
            high: high,
        }
    }
}


impl Pot<f32> for SinePot {
    fn read(&self) -> f32 {
        let sin_t = (&mut *self.gen.borrow_mut())();
        let out = self.low + ((self.high - self.low) * ((1.0 + sin_t) / 2.0));
        println!("out: {}", out);
        out
    }
}


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
