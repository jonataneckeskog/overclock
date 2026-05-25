use tokio::sync::mpsc;

mod core;
mod flow;

pub struct PipeLine<In, Out> {
    pub capacity: usize,
    pub transform:
        Option<Box<dyn FnOnce(mpsc::Receiver<In>, mpsc::Sender<Out>) + Send + Sync + 'static>>,
}

impl PipeLine<(), ()> {
    pub fn new() -> Self {
        PipeLine {
            capacity: 32,
            transform: Some(Box::new(|mut source_rx, final_tx| {
                tokio::spawn(async move {
                    while let Some(item) = source_rx.recv().await {
                        if final_tx.send(item).await.is_err() {
                            break;
                        }
                    }
                });
            })),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        PipeLine {
            capacity,
            transform: Some(Box::new(|mut source_rx, final_tx| {
                tokio::spawn(async move {
                    while let Some(item) = source_rx.recv().await {
                        if final_tx.send(item).await.is_err() {
                            break;
                        }
                    }
                });
            })),
        }
    }
}
