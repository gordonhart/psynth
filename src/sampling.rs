use std::path::Path;

use anyhow::Result;
use hound::WavReader;

use crate::{Sample, Generator};


pub trait SampleTrack {
    fn next(&mut self) -> Option<Sample>;
    fn reset(&mut self);
}

// pub type SampleTrack = Vec<Sample>;
// pub type SampleIter<'a> = std::slice::Iter<'a, Sample>;
pub struct VecTrack {
    track: Vec<Sample>,
    counter: usize,
}

impl VecTrack {
    pub fn from_generator(mut generator: Generator, length: usize) -> Self {
        let mut track = Vec::with_capacity(length);
        for _ in 0 .. length {
            track.push(generator());
        }
        VecTrack {
            track: track,
            counter: 0,
        }
    }

    pub fn try_from_wav_file<P>(filename: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let mut reader = WavReader::open(filename)?;
        println!("{:?}", reader.spec());
        let mut track = Vec::with_capacity(reader.len() as usize);
        for s in reader.samples() {
            track.push(s?);
        }
        Ok(VecTrack {
            track: track,
            counter: 0,
        })
    }
}

impl SampleTrack for VecTrack {
    fn reset(&mut self) {
        self.counter = 0;
    }

    fn next(&mut self) -> Option<Sample> {
        let ret = match self.track.get(self.counter) {
            Some(s) => Some(*s),
            None => None,
        };
        self.counter += 1;
        ret
    }
}
