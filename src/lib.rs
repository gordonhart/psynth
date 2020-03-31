pub mod generators;
pub mod filters;
pub mod consumers;
pub mod observers;


pub type Sample = f32;


/// Source of an audio stream.
///
/// Each call generates the output value at that given instance in time, e.g. for a sample rate of
/// 44100Hz, this function should be called 44100 times per second to generate that second's worth
/// of sound.
pub type Generator = Box<dyn FnMut() -> Sample + Send>;
 

/// Transformation applied to an audio stream.
///
/// A call of a `Filter` applies its transformation to the provided value and returns it. `Filter`s
/// will usually have some internal data structures allowing them to track the passage of time and
/// history of inputs and outputs.
pub type Filter = Box<dyn FnMut(Sample) -> Sample + Send>;


/// End consumer of an audio stream.
///
/// Calls the `Generator` repeatedly to generate the audio stream them does some implementation-
/// specific processing on the data, probably involving packing the provided buffer.
///
/// Audio streams are driven by `Consumer`s. The frequency of calls to the generator are determined
/// by the `Consumer`s need to fill buffers as provided to the `Consumer` by external (`cpal`) code. 
// pub type Consumer = Box<dyn FnMut(Generator) -> Box<dyn FnMut(&mut [Sample]) + Send>>;
pub trait Consumer: Send {
    fn bind(self, generator: Generator) -> Self;
    fn fill(&mut self, output_buffer: &mut [Sample]);
}


pub trait Observer {
    fn sample(&mut self, sample: Sample);
}


/// Consume `self` and the provided `Filter` to create a new `Generator` with the filter applied.
///
/// Exists to provide a better interface to `filters::compose`, enabling the builder pattern:
///
/// ```rust
/// use psynth::{FilterComposable, Generator, filters, generators};
/// let config = cpal::StreamConfig { channels: 1, sample_rate: cpal::SampleRate(44100) };
/// let mut gen: Generator = generators::flat(&config, 440.0)
///     .compose(filters::warble(&config, 1.0))
///     .compose(filters::warble(&config, 2.0));
/// ```
pub trait FilterComposable {
    fn compose(self, filter: Filter) -> Generator;
}


impl FilterComposable for Generator {
    fn compose(self, filter: Filter) -> Generator {
        filters::compose(self, filter)
    }
}
