use std::cell::{Cell, RefCell};

use crate::{Pot, Generator};
use crate::sampling::SampleTrack;


/// A profile, usually on `[0,1]`, usually describing a signal's strength at a given instant in
/// time.
pub trait Curve<T> {
    fn read(&self, sample_index: u64) -> T;
}


impl Curve<f32> for f32 {
    fn read(&self, _: u64) -> f32 {
        *self
    }
}


impl<F> Curve<f32> for F
where
    F: Fn(u64) -> f32,
{
    fn read(&self, idx: u64) -> f32 {
        (self)(idx)
    }
}


/// Device representing a real-world 'key' (or button) on a machine.
///
/// Holds:
///   - The `SampleTrack` it uses to produce sound
///   - A `Pot<bool>` indicating if the switch is open or closed
///   - An attack `Curve<f32>` defining the activation ramp-up behavior when switched on
///   - A sustain `Curve<f32>` defining the deactivation ramp-down behavior when switched off
pub struct SimpleKey<T, P, C1, C2>
where
    T: SampleTrack + Send,
    P: Pot<bool> + Send,
    C1: Curve<f32> + Send,
    C2: Curve<f32> + Send,
{
    track: RefCell<T>,
    active: P,
    attack: C1,
    sustain: C2,
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
    pub fn new(track: T, active: P, attack: C1, sustain: C2) -> Self {
        Self {
            track: RefCell::new(track),
            active,
            attack,
            sustain,
            n_since_activated: Cell::new(0),
            n_since_deactivated: Cell::new(0),
        }
    }

    /// Transform into a `Generator` (consuming).
    pub fn into_generator(self) -> Generator {
        Box::new(move || self.read())
    }
}


/// For maximum compatibility, a `SimpleKey` _could_ be used as a `Pot` input to some other
/// component.
///
/// However, it is expected that a `SimpleKey` will usually be used as a `Generator` via
/// `into_generator`.
impl<T, P, C1, C2> Pot<f32> for SimpleKey<T, P, C1, C2>
where
    T: SampleTrack + Send,
    P: Pot<bool> + Send,
    C1: Curve<f32> + Send,
    C2: Curve<f32> + Send,
{
    fn read(&self) -> f32 {
        let is_active = self.active.read();
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
            next_sample * self.attack.read(n_since_activated_prev)
        } else {
            self.n_since_deactivated.set(n_since_deactivated_prev + 1);
            next_sample * self.sustain.read(n_since_deactivated_prev)
        }
    }
}
