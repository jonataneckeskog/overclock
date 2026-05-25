use tokio::sync::mpsc;

mod core;
mod flow;

pub struct Pipeline<In, Out> {
    pub capacity: usize,
    pub transform:
        Option<Box<dyn FnOnce(mpsc::Receiver<In>, mpsc::Sender<Out>) + Send + Sync + 'static>>,
}

impl<T> Pipeline<T, T>
where
    T: Send + 'static,
{
    pub fn with_capacity(capacity: usize) -> Self {
        Pipeline {
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

    /// The standard default entry point (uses capacity 32)
    pub fn start() -> Self {
        Self::with_capacity(32)
    }
}
