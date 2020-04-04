use crate::music::notes::{Tone, Octave};

//
// TODO: implement
//

pub fn c_major(octave: Octave) -> Vec<Tone> {
    let notes = vec!["C", "D", "E", "F", "G", "A", "B"];
    notes
        .iter()
        .map(|n| Tone::try_from(format!("{}{}", n, octave as i32)).unwrap())
        .collect()
}
