use std::io::{Stdout, Write};

use crate::{Observer, Sample};


/// Dump all sample values to stdout.
impl Observer for Stdout {
    fn sample(&mut self, sample: Sample) {
        self.write_fmt(format_args!("{}\n", sample))
            .expect("failed to write to stdout");
    }
}


// TODO: implement
pub struct WavWriter {}
impl Observer for WavWriter {
    fn sample(&mut self, sample: Sample) {
        unimplemented!()
    }
}
