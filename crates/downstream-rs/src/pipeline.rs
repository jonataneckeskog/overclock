use tokio::sync::mpsc;
use tokio::task::JoinHandle;

pub struct Pipeline<In, Out> {
    pub capacity: usize,
    pub transform: Option<
        Box<
            dyn FnOnce(mpsc::Receiver<In>, mpsc::Sender<Out>, &mut Vec<JoinHandle<()>>)
                + Send
                + Sync
                + 'static,
        >,
    >,
}

impl<T> Pipeline<T, T>
where
    T: Send + 'static,
{
    pub fn with_capacity(capacity: usize) -> Self {
        Pipeline {
            capacity,
            transform: Some(Box::new(|mut source_rx, final_tx, tasks| {
                let handle = tokio::spawn(async move {
                    while let Some(item) = source_rx.recv().await {
                        if final_tx.send(item).await.is_err() {
                            break;
                        }
                    }
                });
                tasks.push(handle);
            })),
        }
    }

    /// The standard default entry point (uses capacity 32)
    pub fn start() -> Self {
        Self::with_capacity(32)
    }
}
