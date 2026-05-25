use futures::StreamExt;

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

    /// Physically connects a fully-constructed Pipeline onto the end of this one.
    pub fn apply<NewOut>(self, other: Pipeline<Out, NewOut>) -> Pipeline<In, NewOut>
    where
        NewOut: Send + 'static,
    {
        let transform_a = self.transform;
        let transform_b = other.transform;
        let capacity = self.capacity;

        Pipeline {
            capacity,
            transform: Some(Box::new(move |source_rx, final_tx| {
                // 1. Create the junction channel between Pipeline A and Pipeline B
                let (mid_tx, mid_rx) = tokio::sync::mpsc::channel::<Out>(capacity);

                // 2. Solder Pipeline A to the junction
                if let Some(a) = transform_a {
                    a(source_rx, mid_tx);
                }

                // 3. Solder Pipeline B to the junction
                if let Some(b) = transform_b {
                    b(mid_rx, final_tx);
                }
            })),
        }
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
    /// Plugs in any generic Stream, fires up execution, and returns a handle to await completion
    pub fn run<S>(mut self, stream: S) -> tokio::task::JoinHandle<()>
    where
        S: futures::Stream<Item = In> + Send + 'static,
    {
        // Create a bridge channel for the stream to feed into
        // It needs to be bridged to tokio since tokio is the runtime
        let (source_tx, source_rx) = tokio::sync::mpsc::channel::<In>(self.capacity);

        tokio::spawn(async move {
            tokio::pin!(stream);

            while let Some(item) = stream.next().await {
                if source_tx.send(item).await.is_err() {
                    break;
                }
            }
        });

        // Connect the end of the pipeline
        let (final_tx, mut final_rx) = tokio::sync::mpsc::channel::<()>(1);

        if let Some(compile) = self.transform.take() {
            compile(source_rx, final_tx);
        }

        // Return the JoinHandle
        tokio::spawn(async move { while final_rx.recv().await.is_some() {} })
    }
}
