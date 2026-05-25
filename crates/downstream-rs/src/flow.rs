use tokio::sync::mpsc;

use crate::PipeLine;

impl<In, Out> PipeLine<In, Out>
where
    In: Send + 'static,
    Out: Send + 'static,
{
    /// Executes a side effect (like saving to a DB) and passes the data through untouched
    pub fn tap<F>(self, action: F) -> PipeLine<In, Out>
    where
        F: Fn(&Out) + Send + Sync + 'static,
    {
        let previous_transform = self.transform;
        // Wrap the closure in an Arc so it can be safely shared/moved into the async task
        let action = std::sync::Arc::new(action);

        PipeLine {
            transform: Some(Box::new(move |source_rx, final_tx| {
                let (mid_tx, mut mid_rx) = tokio::sync::mpsc::channel::<Out>(32);

                if let Some(prev) = previous_transform {
                    prev(source_rx, mid_tx);
                }

                let action_clone = action.clone();
                tokio::spawn(async move {
                    while let Some(item) = mid_rx.recv().await {
                        // Execute the side effect
                        action_clone(&item);

                        // Pass the item through untouched
                        if final_tx.send(item).await.is_err() {
                            return; // Downstream is dead
                        }
                    }
                });
            })),
        }
    }

    /// Zips another stream into this one, combining both inputs into a new output
    pub fn join<Other, NewOut, F>(
        self,
        mut other_rx: mpsc::Receiver<Other>,
        combine: F,
    ) -> PipeLine<In, NewOut>
    where
        Other: Send + 'static,
        NewOut: Send + 'static,
        F: Fn(Out, Other) -> NewOut + Send + Sync + 'static,
    {
        let previous_transform = self.transform;
        let combine = std::sync::Arc::new(combine);

        PipeLine {
            transform: Some(Box::new(move |source_rx, final_tx| {
                let (mid_tx, mut mid_rx) = tokio::sync::mpsc::channel::<Out>(32);

                if let Some(prev) = previous_transform {
                    prev(source_rx, mid_tx);
                }

                let combine_clone = combine.clone();
                tokio::spawn(async move {
                    // Pull from both streams simultaneously
                    while let Some(item_self) = mid_rx.recv().await {
                        if let Some(item_other) = other_rx.recv().await {
                            let combined_output = combine_clone(item_self, item_other);

                            if final_tx.send(combined_output).await.is_err() {
                                return; // Downstream is dead
                            }
                        } else {
                            // The 'other' receiver closed, so we can't zip anymore
                            return;
                        }
                    }
                });
            })),
        }
    }

    /// Closes the downstream connection after passing exactly `limit` items
    pub fn take(self, limit: usize) -> PipeLine<In, Out> {
        let previous_transform = self.transform;

        PipeLine {
            transform: Some(Box::new(move |source_rx, final_tx| {
                // If limit is 0, don't even bother wiring up the rest of the stream
                if limit == 0 {
                    return;
                }

                let (mid_tx, mut mid_rx) = tokio::sync::mpsc::channel::<Out>(32);

                if let Some(prev) = previous_transform {
                    prev(source_rx, mid_tx);
                }

                tokio::spawn(async move {
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
            })),
        }
    }

    /// Emits a fixed-size ring buffer of the last `size` items, rolling forward with every new event.
    pub fn window(self, size: usize) -> PipeLine<In, std::collections::VecDeque<Out>>
    where
        Out: Clone + Send + 'static,
    {
        let previous_transform = self.transform;

        PipeLine {
            transform: Some(Box::new(move |source_rx, final_tx| {
                let (mid_tx, mut mid_rx) = tokio::sync::mpsc::channel::<Out>(32);

                if let Some(prev) = previous_transform {
                    prev(source_rx, mid_tx);
                }

                tokio::spawn(async move {
                    let mut buffer = std::collections::VecDeque::with_capacity(size);

                    while let Some(item) = mid_rx.recv().await {
                        buffer.push_back(item);

                        // If the window size is exceeded, pop the oldest item
                        if buffer.len() > size {
                            buffer.pop_front();
                        }

                        // Emit a clone of the current window state downstream
                        if final_tx.send(buffer.clone()).await.is_err() {
                            return; // Downstream is dead, exit task.
                        }
                    }
                });
            })),
        }
    }

    /// Batches exactly `size` incoming items into a `Vec<Out>` before sending them downstream, dropping any leftover trailing items if the stream ends early.
    pub fn chunk(self, size: usize) -> PipeLine<In, Vec<Out>> {
        let previous_transform = self.transform;

        PipeLine {
            transform: Some(Box::new(move |source_rx, final_tx| {
                let (mid_tx, mut mid_rx) = tokio::sync::mpsc::channel::<Out>(32);

                if let Some(prev) = previous_transform {
                    prev(source_rx, mid_tx);
                }

                tokio::spawn(async move {
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
            })),
        }
    }
}
