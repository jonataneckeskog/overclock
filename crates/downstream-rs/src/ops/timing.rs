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
            transform: Some(Box::new(move |source_rx, final_tx, tasks| {
                let (mid_tx, mut mid_rx) = tokio::sync::mpsc::channel::<Out>(baseline_capacity);

                if let Some(prev) = previous_transform {
                    prev(source_rx, mid_tx, tasks);
                }

                let handle = tokio::spawn(async move {
                    while let Some(item) = mid_rx.recv().await {
                        // Natively sleep for the floating-point duration
                        tokio::time::sleep(delay).await;

                        if final_tx.send(item).await.is_err() {
                            break;
                        }
                    }
                });
                tasks.push(handle);
            })),
        }
    }

    /// Emits a fixed-size ring buffer of the last `size` items, rolling forward with every new event.
    pub fn window(self, size: usize) -> Pipeline<In, std::collections::VecDeque<Out>>
    where
        Out: Clone + Send + 'static,
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
                tasks.push(handle);
            })),
        }
    }
}
