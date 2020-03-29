use std::io::{Stdout, Write};

use crate::{Observer, Sample};


impl Observer for Stdout {
    fn sample(&mut self, sample: Sample) {
        self.write_fmt(format_args!("{}\n", sample))
            .expect("failed to write to stdout");
    }
}
