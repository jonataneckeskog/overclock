use crate::Pipeline;

impl<In, Out> Pipeline<In, Out>
where
    In: Send + 'static,
    Out: Send + 'static,
{
    /// Executes a side effect (like logging or saving to a DB) and passes the data through untouched
    pub fn tap<F>(self, action: F) -> Pipeline<In, Out>
    where
        F: Fn(&Out) + Send + Sync + 'static,
    {
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
                    while let Some(item) = mid_rx.recv().await {
                        // Action is fully owned by this task now, no Arc needed.
                        action(&item);

                        if final_tx.send(item).await.is_err() {
                            return;
                        }
                    }
                });
                tasks.push(handle);
            })),
        }
    }

    /// Closes the downstream connection after passing exactly `limit` items
    pub fn take(self, limit: usize) -> Pipeline<In, Out> {
        let previous_transform = self.transform;
        let capacity = self.capacity;

        Pipeline {
            capacity,
            transform: Some(Box::new(move |source_rx, final_tx, tasks| {
                // If limit is 0, don't even bother wiring up the rest of the stream
                if limit == 0 {
                    return;
                }

                let (mid_tx, mut mid_rx) = tokio::sync::mpsc::channel::<Out>(capacity);

                if let Some(prev) = previous_transform {
                    prev(source_rx, mid_tx, tasks);
                }

                let handle = tokio::spawn(async move {
                    let mut count = 0;

                    while let Some(item) = mid_rx.recv().await {
                        if final_tx.send(item).await.is_err() {
                            return; // Downstream is dead
                        }

                        count += 1;
                        if count >= limit {
                            break; // Stop receiving, closing the channel drops upstream
                        }
                    }
                });
                tasks.push(handle);
            })),
        }
    }
}
