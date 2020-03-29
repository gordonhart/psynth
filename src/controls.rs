use crate::{Generator, Consumer};


/// Given a single-channel generator, duplicate its output to two different consumers.
// TODO: is there a way to use a `where` clause here to avoid all of this repetition?
pub fn tee(
    generator: Generator,
    consumer_a: Consumer,
    consumer_b: Consumer,
) -> Consumer {

    /*
    const BUFSIZE: usize = 1_000_000;
    let mut buffer = Box::new([0.0f32; BUFSIZE]);

    move |data: &mut [f32]| {
        unimplemented!()
    }
    */
    unimplemented!()
}
