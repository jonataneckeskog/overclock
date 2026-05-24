use tokio::sync::mpsc;

mod core;
mod flow;

pub struct PipeLine<In, Out> {
    pub transform:
        Option<Box<dyn FnOnce(mpsc::Receiver<In>, mpsc::Sender<Out>) + Send + Sync + 'static>>,
}

impl PipeLine<(), ()> {
    pub fn new() -> Self {
        panic!("TODO: Initialize an empty PipeLine blueprint");
    }
}
