//! Various ways to multiplex streams.

use std::sync::{Arc, Mutex};

use crate::{Generator, Sample, Pot};


/// Multiplex together left/right streams in a stereo setup using a custom mux function.
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


/// Adjust the left/right balance of two streams.
///
/// Values yielded from the `balance_pot` should fall within `[-1,1]` where `-1` corresponds to all
/// sound routed to the left channel, `1` sees all sound to the right channel, and `0` has no effect
/// on balance.
///
/// The two output `Generator`s are entangled (similar to those yielded from `mux2`) and should be
/// used in tandem at the same rate.
pub fn balance<P>(
    balance_pot: P,
    left: Generator,
    right: Generator,
) -> (Generator, Generator)
where
    P: Pot<f32> + 'static,
{
    let balance_fun = move |l, r| {
        let balance = balance_pot.read();
        let l_out = l * (1.0 - balance).min(1.0) + r * (-balance).max(0.0);
        let r_out = l * balance.max(0.0) + r * (balance + 1.0).min(1.0);
        (l_out, r_out)
    };
    mux2(balance_fun, left, right)
}
