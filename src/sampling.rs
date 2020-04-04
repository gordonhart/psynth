use std::path::Path;

use anyhow::{anyhow, Result};
use hound::{WavReader, WavSpec, SampleFormat};

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

    pub fn try_from_wav_file<P>(sample_rate: u32, filename: P) -> Result<Self>
    where
        P: AsRef<Path> + Clone,
    {
        let mut reader = WavReader::open(filename.clone())?;
        let mut track = Vec::with_capacity(reader.len() as usize);

        let spec = reader.spec();
        println!("{:?}", spec);
        let filename_str = filename.as_ref().to_str().unwrap_or_else(|| "<error parsing filename>");
        if spec.sample_rate != sample_rate {
            return Err(anyhow!(
                "unable to handle WAV sample rate '{}' that differs from output sample rate '{}' \
                for file '{:?}'", spec.sample_rate, sample_rate, filename_str
            ));
        }
        match spec {
            WavSpec { sample_format: SampleFormat::Float, bits_per_sample: 32, .. } => {
                for s in reader.samples() {
                    track.push(s?);
                }
            },
            WavSpec { sample_format: SampleFormat::Int, bits_per_sample: 16, .. } => {
                for s in reader.samples::<i16>() {
                    let int_sample = s?;
                    track.push((int_sample as f32) / (std::i16::MAX as f32));
                }
            },
            other => {
                return Err(anyhow!(
                    "unable to handle '{:?}' WAV spec for '{:?}'", other, filename_str
                ));
            },
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


/// For compatibility's sake.
impl SampleTrack for Generator {
    fn next(&mut self) -> Option<Sample> {
        Some(self())
    }
    fn reset(&mut self) {
        // no resetting a Generator
    }
}
