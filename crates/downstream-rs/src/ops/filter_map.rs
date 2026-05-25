use crate::Pipeline;

impl<In, Out> Pipeline<In, Out>
where
    In: Send + 'static,
    Out: Send + 'static,
{
    /// Transforms the data from type `Out` to type `NewOut`
    pub fn pipe<F, NewOut>(self, transform: F) -> Pipeline<In, NewOut>
    where
        // The closure MUST return an Option!
        F: Fn(Out) -> Option<NewOut> + Send + Sync + 'static,
        NewOut: Send + 'static,
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
                        if let Some(new_item) = transform(item) {
                            if final_tx.send(new_item).await.is_err() {
                                break;
                            }
                        }
                    }
                });
                tasks.push(handle);
            })),
        }
    }

    /// Transforms data in parallel using a worker pool.
    ///
    /// **Warning:** This operator does NOT preserve item ordering.
    /// It automatically scales the number of workers based on available CPU cores.
    pub fn par_pipe<F, NewOut>(self, transform: F) -> Pipeline<In, NewOut>
    where
        F: Fn(Out) -> Option<NewOut> + Send + Sync + 'static,
        NewOut: Send + 'static,
    {
        let previous_transform = self.transform;
        let capacity = self.capacity;

        Pipeline {
            capacity,
            transform: Some(Box::new(move |source_rx, final_tx, tasks| {
                let (mid_tx, mid_rx) = tokio::sync::mpsc::channel::<Out>(capacity);

                if let Some(prev) = previous_transform {
                    prev(source_rx, mid_tx, tasks);
                }

                let num_workers = std::thread::available_parallelism()
                    .map(|n| n.get())
                    .unwrap_or(1);

                let shared_rx = std::sync::Arc::new(tokio::sync::Mutex::new(mid_rx));
                let transform = std::sync::Arc::new(transform);

                for _ in 0..num_workers {
                    let rx = shared_rx.clone();
                    let tx = final_tx.clone();
                    let transform = transform.clone();

                    let handle = tokio::spawn(async move {
                        loop {
                            let item = {
                                let mut guard = rx.lock().await;
                                guard.recv().await
                            };

                            match item {
                                Some(data) => {
                                    if let Some(new_item) = transform(data) {
                                        if tx.send(new_item).await.is_err() {
                                            break;
                                        }
                                    }
                                }
                                None => break,
                            }
                        }
                    });
                    tasks.push(handle);
                }
            })),
        }
    }
}
