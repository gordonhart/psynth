pub mod music;
pub mod generator;
pub mod filter;
pub mod consumer;
pub mod observer;
pub mod control;
pub mod sampling;
pub mod device;


/// Audio out value at a given instant.
///
/// Should remain on `[-1, 1]` or else the playback device enters undefined behavior that probably
/// involves clipping.
pub type Sample = f32;


/// Source of an audio stream.
///
/// Each call generates the output value at that given instance in time, e.g. for a sample rate of
/// 44100Hz, this function should be called 44100 times per second to generate that second's worth
/// of sound.
// NOTE: current priorities are functionality and ease of use over corectness, 'static it is
// pub type Generator<'a> = Box<dyn FnMut() -> Sample + Send + 'a>;
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
pub trait Consumer: Send {
    fn fill(&mut self, output_buffer: &mut [Sample]);
}


/// Passive observer on the stream received by a `Consumer`.
pub trait Observer {
    fn sample(&mut self, sample: Sample);
}


/// A potentiometer provides a controllable input to a function.
pub trait Pot<T>: Send {

    /// Read a value off of the `Pot`.
    ///
    /// Note that the reference to `&self` is immutable -- `Pot` implementors shouldn't really be
    /// modifying themselves based on reads, as that goes against their meatspace namesake, which
    /// is not altered by the act of reading.
    fn read(&self) -> T;
}


/// Consume `self` and the provided `Filter` to create a new `Generator` with the filter applied.
///
/// Exists to provide a better interface to `filters::compose`, enabling the builder pattern:
///
/// ```rust
/// use psynth::{FilterComposable, Generator, filters, generators};
/// let config = cpal::StreamConfig { channels: 1, sample_rate: cpal::SampleRate(44100) };
/// let mut gen: Generator = generators::sine(config.sample_rate.0, 440.0)
///     .compose(filters::gain(0.5))
///     .compose(filters::offset(2.0));
/// ```
pub trait FilterComposable {
    fn compose(self, filter: Filter) -> Generator;
    fn fork<F>(self, join_function: F) -> Generator
    where
        F: FnMut(Generator, Generator) -> Generator + Send;
}


impl FilterComposable for Generator {
    fn compose(self, filter: Filter) -> Generator {
        filter::compose(self, filter)
    }

    fn fork<F>(self, mut join_function: F) -> Generator
    where
        F: FnMut(Generator, Generator) -> Generator + Send,
    {
        let (left_gen, right_gen) = control::flow::fork(self);
        join_function(left_gen, right_gen)
    }
}
