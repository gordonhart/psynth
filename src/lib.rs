pub mod generators;
pub mod filters;
pub mod consumers;


pub type Sample = f32;


/// Source of an audio stream.
///
/// Each call generates the output value at that given instance in time, e.g. for a sample rate of
/// 44100Hz, this function should be called 44100 times per second to generate that second's worth
/// of sound.
pub type Generator = Box<dyn FnMut() -> Sample + Send>;
 

/// Transformation applied to an audio stream.
///
/// A call of a `Filter` calls the connected `Generator`, applies its transformation to the value
/// received, and returns it.
pub type Filter = Box<dyn FnMut(&mut Generator) -> Sample + Send>;


/// End consumer of an audio stream.
///
/// Calls the `Generator` repeatedly to generate the audio stream them does some implementation-
/// specific processing on the data.
pub type Consumer = Box<dyn FnMut(&mut Generator, &mut [Sample]) + Send>;
