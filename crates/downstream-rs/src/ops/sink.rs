use crate::Pipeline;

impl<In, Out> Pipeline<In, Out>
where
    In: Send + 'static,
    Out: Send + 'static,
{
    /// Caps the pipeline with a terminal action.
    pub fn sink<F>(self, action: F) -> Pipeline<In, ()>
    where
        F: Fn(Out) + Send + Sync + 'static,
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
                        action(item);
                    }
                    drop(final_tx);
                });
                tasks.push(handle);
            })),
        }
    }

    /// Converts the pipeline into a Receiver, allowing manual consumption.
    pub fn into_stream(
        mut self,
        source_rx: tokio::sync::mpsc::Receiver<In>,
        tasks: &mut Vec<tokio::task::JoinHandle<()>>,
    ) -> tokio::sync::mpsc::Receiver<Out> {
        let (final_tx, final_rx) = tokio::sync::mpsc::channel::<Out>(self.capacity);

        if let Some(compile) = self.transform.take() {
            compile(source_rx, final_tx, tasks);
        }

        final_rx
    }
}

impl<In> Pipeline<In, ()>
where
    In: Send + 'static,
{
    /// Executes the pipeline using the provided Stream as a source.
    /// 
    /// This method awaits the completion of all internal tasks, ensuring that
    /// side effects and branches have finished before returning.
    pub async fn run<S>(mut self, stream: S) -> Result<(), tokio::task::JoinError>
    where
        S: futures::Stream<Item = In> + Send + 'static,
    {
        use futures::StreamExt;
        let mut tasks = Vec::new();

        // Create a bridge channel for the stream to feed into
        let (source_tx, source_rx) = tokio::sync::mpsc::channel::<In>(self.capacity);

        let stream_handle = tokio::spawn(async move {
            tokio::pin!(stream);

            while let Some(item) = stream.next().await {
                if source_tx.send(item).await.is_err() {
                    break;
                }
            }
        });
        tasks.push(stream_handle);

        // Connect the end of the pipeline
        let (final_tx, mut final_rx) = tokio::sync::mpsc::channel::<()>(1);

        if let Some(compile) = self.transform.take() {
            compile(source_rx, final_tx, &mut tasks);
        }

        // Exhaust the final channel
        let exhaust_handle = tokio::spawn(async move { while final_rx.recv().await.is_some() {} });
        tasks.push(exhaust_handle);

        // Await all tasks in the pipeline
        for task in tasks {
            task.await?;
        }

        Ok(())
    }
}
