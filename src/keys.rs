use std::cell::{Cell, RefCell};

use crate::{Pot, Generator};
use crate::sampling::SampleTrack;


pub trait Curve<T> {
    fn read(&self, sample_index: u64) -> T;
}


impl Curve<f32> for f32 {
    fn read(&self, _: u64) -> f32 {
        *self
    }
}


pub struct SimpleKey<T, P, C1, C2>
where
    T: SampleTrack + Send,
    P: Pot<bool> + Send,
    C1: Curve<f32> + Send,
    C2: Curve<f32> + Send,
{
    track: RefCell<T>,
    activate: P, // is the key pressed?
    velocity: C1, // curve describing ramp up after press
    sustain: C2, // curve describing dropoff after release
    n_since_activated: Cell<u64>,
    n_since_deactivated: Cell<u64>,
}

impl<T, P, C1, C2> SimpleKey<T, P, C1, C2>
where
    T: SampleTrack + Send + 'static,
    P: Pot<bool> + Send + 'static,
    C1: Curve<f32> + Send + 'static,
    C2: Curve<f32> + Send + 'static,
{
    pub fn new(track: T, activate: P, velocity: C1, sustain: C2) -> Self {
        Self {
            track: RefCell::new(track),
            activate,
            velocity,
            sustain,
            n_since_activated: Cell::new(0),
            n_since_deactivated: Cell::new(0),
        }
    }

    pub fn into_generator(self) -> Generator {
        Box::new(move || self.read())
    }
}


impl<T, P, C1, C2> Pot<f32> for SimpleKey<T, P, C1, C2>
where
    T: SampleTrack + Send,
    P: Pot<bool> + Send,
    C1: Curve<f32> + Send,
    C2: Curve<f32> + Send,
{
    fn read(&self) -> f32 {
        let is_active = self.activate.read();
        let n_since_activated_prev = self.n_since_activated.get();
        let n_since_deactivated_prev = self.n_since_deactivated.get();
        let mut track_ref = self.track.borrow_mut();

        // just turned on
        if is_active && n_since_activated_prev == 0 {
            track_ref.reset();
            self.n_since_deactivated.set(0);
        } else if !is_active && n_since_deactivated_prev == 0 {
            self.n_since_activated.set(0);
        }

        let next_sample = track_ref.next().unwrap_or(0.0);

        if is_active {
            self.n_since_activated.set(n_since_activated_prev + 1);
            next_sample * self.velocity.read(n_since_activated_prev)
        } else {
            self.n_since_deactivated.set(n_since_deactivated_prev + 1);
            next_sample * self.sustain.read(n_since_deactivated_prev)
        }
    }
}
