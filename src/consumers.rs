use crate::{Sample, Consumer, Generator, Observer};


/// Write the output stream as generated from the `next_sample` function.
///
/// All channels of the output stream are written with the same data.
// TODO: add example usage
pub fn write_output_stream_mono(channels: usize) -> Consumer {
    Box::new(move |generator: &mut Generator, output: &mut [Sample]| {
        for frame in output.chunks_mut(channels) {
            // TODO: reintroduce value type parameterization from example
            let value = generator();
            for sample in frame.iter_mut() {
                *sample = value;
            }
        }
    })
}


pub fn write_output_stream_mono_with_observers(
    channels: usize,
    mut observers: Vec<Box<dyn Observer + Send>>,
) -> Consumer {
    Box::new(move |generator: &mut Generator, output: &mut [Sample]| {
        for frame in output.chunks_mut(channels) {
            let value = generator();
            for sample in frame.iter_mut() {
                *sample = value;
            }
            for observer in observers.iter_mut() {
                observer.sample(value);
            }
        }
    })
}
