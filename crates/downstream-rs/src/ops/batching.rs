use crate::Pipeline;

impl<In, Out> Pipeline<In, Out>
where
    In: Send + 'static,
    Out: Send + 'static,
{
    /// Batches exactly `size` incoming items into a `Vec<Out>` before sending them downstream, dropping any leftover trailing items if the stream ends early.
    pub fn chunk(self, size: usize) -> Pipeline<In, Vec<Out>> {
        let previous_transform = self.transform;
        let capacity = self.capacity;

        Pipeline {
            capacity,
            transform: Some(Box::new(move |source_rx, final_tx, tasks| {
                let (mid_tx, mut mid_rx) = tokio::sync::mpsc::channel::<Out>(capacity);

                if let Some(prev) = previous_transform {
                    prev(source_rx, mid_tx, tasks);
                }

                let handle = tokio::spawn(async move {
                    // Hold a vector in memory inside this task
                    let mut chunk = Vec::with_capacity(size);

                    while let Some(item) = mid_rx.recv().await {
                        chunk.push(item);

                        //  Once it hits the limit, fire it downstream
                        if chunk.len() == size {
                            if final_tx.send(chunk).await.is_err() {
                                return; // Downstream is dead, exit task.
                            }
                            // Reset the state for the next batch
                            chunk = Vec::with_capacity(size);
                        }
                    }
                });
                tasks.push(handle);
            })),
        }
    }

    /// Acts as a dedicated shock-absorber stage with a custom queue size
    pub fn buffer(self, size: usize) -> Pipeline<In, Out> {
        let previous_transform = self.transform;
        let baseline_capacity = self.capacity;

        Pipeline {
            capacity: baseline_capacity, // Preserve the default for downstream
            transform: Some(Box::new(move |source_rx, final_tx, tasks| {
                // This specific stage gets the massive capacity
                let (mid_tx, mut mid_rx) = tokio::sync::mpsc::channel::<Out>(size);

                if let Some(prev) = previous_transform {
                    prev(source_rx, mid_tx, tasks);
                }

                let handle = tokio::spawn(async move {
                    while let Some(item) = mid_rx.recv().await {
                        if final_tx.send(item).await.is_err() {
                            break;
                        }
                    }
                });
                tasks.push(handle);
            })),
        }
    }

    /// Permanently changes the default capacity for all subsequent channels
    pub fn set_capacity(mut self, size: usize) -> Self {
        self.capacity = size;
        self
    }
}
