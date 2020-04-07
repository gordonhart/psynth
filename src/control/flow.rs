use crate::{generator, filter, Pot, Generator};
use crate::control::mux;


/// Fork the provided `Generator` into two entangled `Generator`s that will yield the same value
/// on each at any given instant.
pub fn fork(generator: Generator) -> (Generator, Generator) {
    mux::mux2(move |l, _| (l, l), generator, generator::silence())
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
    let mut clipper = filter::clip(-1.0, 1.0);
    Box::new(move || {
        let l_val = left();
        let r_val = right();
        let b_val = clipper(bias.read()) + 1.0;
        ((2.0 - b_val) * 0.5 * l_val) + (b_val * 0.5 * r_val)
    })
}
