use crate::Pipeline;

impl<In, Out> Pipeline<In, Out>
where
    In: Send + 'static,
    Out: Send + 'static,
{
    /// Halts execution for the specified number of seconds (as a float) before passing each item downstream
    pub fn wait(self, seconds: f64) -> Pipeline<In, Out> {
        let previous_transform = self.transform;
        let baseline_capacity = self.capacity;

        // Convert the floating-point seconds into a standard Duration upfront
        let delay = std::time::Duration::from_secs_f64(seconds);

        Pipeline {
            capacity: baseline_capacity,
            transform: Some(Box::new(move |source_rx, final_tx| {
                let (mid_tx, mut mid_rx) = tokio::sync::mpsc::channel::<Out>(baseline_capacity);

                if let Some(prev) = previous_transform {
                    prev(source_rx, mid_tx);
                }

                tokio::spawn(async move {
                    while let Some(item) = mid_rx.recv().await {
                        // Natively sleep for the floating-point duration
                        tokio::time::sleep(delay).await;

                        if final_tx.send(item).await.is_err() {
                            break;
                        }
                    }
                });
            })),
        }
    }

    /// Executes a side effect (like logging or saving to a DB) and passes the data through untouched
    pub fn tap<F>(self, action: F) -> Pipeline<In, Out>
    where
        F: Fn(&Out) + Send + Sync + 'static,
    {
        let previous_transform = self.transform;

        Pipeline {
            capacity: self.capacity,
            transform: Some(Box::new(move |source_rx, final_tx| {
                let (mid_tx, mut mid_rx) = tokio::sync::mpsc::channel::<Out>(self.capacity);

                if let Some(prev) = previous_transform {
                    prev(source_rx, mid_tx);
                }

                tokio::spawn(async move {
                    while let Some(item) = mid_rx.recv().await {
                        // Action is fully owned by this task now, no Arc needed.
                        action(&item);

                        if final_tx.send(item).await.is_err() {
                            return;
                        }
                    }
                });
            })),
        }
    }

    /// Zips another stream into this one, combining both inputs into a new output
    pub fn join<Other, NewOut, F>(
        self,
        mut other_rx: tokio::sync::mpsc::Receiver<Other>,
        combine: F,
    ) -> Pipeline<In, NewOut>
    where
        Other: Send + 'static,
        NewOut: Send + 'static,
        F: Fn(Out, Other) -> NewOut + Send + Sync + 'static,
    {
        let previous_transform = self.transform;

        Pipeline {
            capacity: self.capacity,
            transform: Some(Box::new(move |source_rx, final_tx| {
                let (mid_tx, mut mid_rx) = tokio::sync::mpsc::channel::<Out>(self.capacity);

                if let Some(prev) = previous_transform {
                    prev(source_rx, mid_tx);
                }

                tokio::spawn(async move {
                    while let Some(item_self) = mid_rx.recv().await {
                        if let Some(item_other) = other_rx.recv().await {
                            // Combine is fully owned by the task, no Arc needed.
                            let combined_output = combine(item_self, item_other);

                            if final_tx.send(combined_output).await.is_err() {
                                return;
                            }
                        } else {
                            return; // The 'other' stream closed
                        }
                    }
                });
            })),
        }
    }

    /// Closes the downstream connection after passing exactly `limit` items
    pub fn take(self, limit: usize) -> Pipeline<In, Out> {
        let previous_transform = self.transform;

        Pipeline {
            capacity: self.capacity,
            transform: Some(Box::new(move |source_rx, final_tx| {
                // If limit is 0, don't even bother wiring up the rest of the stream
                if limit == 0 {
                    return;
                }

                let (mid_tx, mut mid_rx) = tokio::sync::mpsc::channel::<Out>(self.capacity);

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
    pub fn window(self, size: usize) -> Pipeline<In, std::collections::VecDeque<Out>>
    where
        Out: Clone + Send + 'static,
    {
        let previous_transform = self.transform;

        Pipeline {
            capacity: self.capacity,
            transform: Some(Box::new(move |source_rx, final_tx| {
                let (mid_tx, mut mid_rx) = tokio::sync::mpsc::channel::<Out>(self.capacity);

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
    pub fn chunk(self, size: usize) -> Pipeline<In, Vec<Out>> {
        let previous_transform = self.transform;

        Pipeline {
            capacity: self.capacity,
            transform: Some(Box::new(move |source_rx, final_tx| {
                let (mid_tx, mut mid_rx) = tokio::sync::mpsc::channel::<Out>(self.capacity);

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

    /// Acts as a dedicated shock-absorber stage with a custom queue size
    pub fn buffer(self, size: usize) -> Pipeline<In, Out> {
        let previous_transform = self.transform;
        let baseline_capacity = self.capacity;

        Pipeline {
            capacity: baseline_capacity, // Preserve the default for downstream
            transform: Some(Box::new(move |source_rx, final_tx| {
                // This specific stage gets the massive capacity
                let (mid_tx, mut mid_rx) = tokio::sync::mpsc::channel::<Out>(size);

                if let Some(prev) = previous_transform {
                    prev(source_rx, mid_tx);
                }

                tokio::spawn(async move {
                    while let Some(item) = mid_rx.recv().await {
                        if final_tx.send(item).await.is_err() {
                            break;
                        }
                    }
                });
            })),
        }
    }

    /// Permanently changes the default capacity for all subsequent channels
    pub fn set_capacity(mut self, size: usize) -> Self {
        self.capacity = size;
        self
    }
}
