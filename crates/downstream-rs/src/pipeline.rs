use tokio::sync::mpsc;
use tokio::task::JoinHandle;

/// A push-based asynchronous data pipeline.
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
    /// Creates a new identity pipeline.
    ///
    /// The `capacity` dictates the internal channel buffer size:
    /// - **Higher capacity** increases memory usage but maximizes throughput by reducing
    ///   task scheduling overhead and context switching.
    /// - **Lower capacity** tightly bounds memory footprint but triggers aggressive backpressure,
    ///   forcing upstream stages to pause frequently if downstream slows down.
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

    /// The standard entry point with a default capacity of 32.
    pub fn start() -> Self {
        Self::with_capacity(32)
    }
}
