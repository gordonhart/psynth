use crate::{
    Sample,
    Consumer,
    Generator,
    Observer,
};


pub struct MonoConsumer {
    channels: usize,
    generator: Option<Generator>,
    observer_channel: Option<std::sync::mpsc::Sender<Sample>>,
    observer_thread_handle: Option<std::thread::JoinHandle<Sample>>,
}


impl MonoConsumer {
    pub fn new(channels: usize) -> Self {
        Self {
            channels: channels,
            generator: None,
            observer_channel: None,
            observer_thread_handle: None,
        }
    }

    // TODO: make part of the `Consumer` trait? add new `Observable` trait? keep here?
    pub fn bind_observers(mut self, mut observers: Vec<Box<dyn Observer + Send>>) -> Self {
        let (sender, receiver) = std::sync::mpsc::channel();
        self.observer_thread_handle = Some(std::thread::spawn(move || loop {
            let sample = receiver.recv().expect("channel closed");
            for observer in observers.iter_mut() {
                observer.sample(sample);
            }
        }));
        self.observer_channel = Some(sender);
        self
    }
}


impl Consumer for MonoConsumer {
    fn bind(mut self, generator: Generator) -> Self {
        self.generator = Some(generator);
        self
    }

    /// Fill the provided the output buffer as generated by the bound `Generator`.
    ///
    /// All channels of the output stream are written with the same data.
    fn fill(&mut self, output_buffer: &mut [Sample]) {
        // TODO: not crazy about the verbosity of this technique used to access self.generator
        match &mut self.generator {
            Some(ref mut gen) => {
                for frame in output_buffer.chunks_mut(self.channels) {
                    // TODO: reintroduce value type parameterization from example
                    let sample = gen();
                    for location in frame.iter_mut() {
                        *location = sample;
                    }
                    if let Some(ref sender) = &self.observer_channel {
                        sender.send(sample).expect("channel closed");
                    }
                }
            },
            None => panic!("`fill` called on unbound `Consumer`"),
        }
    }
}
