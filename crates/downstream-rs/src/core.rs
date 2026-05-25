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

        Pipeline {
            capacity: self.capacity,
            transform: Some(Box::new(move |source_rx, final_tx| {
                let (mid_tx, mut mid_rx) = tokio::sync::mpsc::channel::<Out>(self.capacity);

                if let Some(prev) = previous_transform {
                    prev(source_rx, mid_tx);
                }

                tokio::spawn(async move {
                    while let Some(item) = mid_rx.recv().await {
                        if let Some(new_item) = transform(item) {
                            if final_tx.send(new_item).await.is_err() {
                                break;
                            }
                        }
                        // If it returned None, the loop instantly restarts and pulls the next item
                    }
                });
            })),
        }
    }

    /// Allows free-form composition by passing the pipeline through a custom function
    pub fn apply<F, NewOut>(self, custom_stage: F) -> Pipeline<In, NewOut>
    where
        F: FnOnce(Self) -> Pipeline<In, NewOut>,
    {
        custom_stage(self)
    }

    /// Caps the pipeline with a final action, returning an executable blueprint
    pub fn sink<F>(self, action: F) -> Pipeline<In, ()>
    where
        F: Fn(Out) + Send + Sync + 'static,
    {
        let previous_transform = self.transform;
        let capacity = self.capacity;

        Pipeline {
            capacity,
            transform: Some(Box::new(move |source_rx, final_tx| {
                let (mid_tx, mut mid_rx) = tokio::sync::mpsc::channel::<Out>(capacity);

                if let Some(prev) = previous_transform {
                    prev(source_rx, mid_tx);
                }

                tokio::spawn(async move {
                    while let Some(item) = mid_rx.recv().await {
                        action(item);
                    }
                    drop(final_tx);
                });
            })),
        }
    }

    // Leaves the exhaust pipe open and hands out the live wire
    pub fn into_stream(
        mut self,
        source_rx: tokio::sync::mpsc::Receiver<In>,
    ) -> tokio::sync::mpsc::Receiver<Out> {
        let (final_tx, final_rx) = tokio::sync::mpsc::channel::<Out>(self.capacity);

        if let Some(compile) = self.transform.take() {
            compile(source_rx, final_tx);
        }

        final_rx
    }
}

impl<In> Pipeline<In, ()>
where
    In: Send + 'static,
{
    /// Plugs in the raw data source, fires up execution, and returns a handle to await completion
    pub fn run(
        mut self,
        source_rx: tokio::sync::mpsc::Receiver<In>,
    ) -> tokio::task::JoinHandle<()> {
        let (final_tx, mut final_rx) = tokio::sync::mpsc::channel::<()>(1);

        if let Some(compile) = self.transform.take() {
            compile(source_rx, final_tx);
        }

        // Return the JoinHandle instead of dropping it
        tokio::spawn(async move { while final_rx.recv().await.is_some() {} })
    }
}
