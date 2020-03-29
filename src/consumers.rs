use crate::{Sample, Consumer, Generator};


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


/// `Consumer` implementation that dumps the data to `stdout`.
// TODO: not crazy about requiring an output buffer be passed in here -- is it possible to change
// the signature of `Consumer` to not assume anything about the consumer (i.e. that it needs/wants
// an output buffer?)
pub fn stdout_dumper() -> Consumer {
    Box::new(move |generator: &mut Generator, output: &mut [Sample]| {
        for frame in output.chunks_mut(1) {
            println!("{}", generator());
        }
    })
}
