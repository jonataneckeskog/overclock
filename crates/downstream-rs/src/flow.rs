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
        F: Fn(&Out) + Send + Sync + 'static, // Passing by reference is usually best for taps!
    {
        panic!("TODO: Implement tap/side-effect logic");
    }

    /// Zips another stream into this one, combining both inputs into a new output
    pub fn join<Other, NewOut, F>(
        self,
        other_rx: mpsc::Receiver<Other>,
        combine: F,
    ) -> PipeLine<In, NewOut>
    where
        Other: Send + 'static,
        NewOut: Send + 'static,
        F: Fn(Out, Other) -> NewOut + Send + Sync + 'static,
    {
        panic!("TODO: Implement async join/zip logic");
    }

    /// Closes the downstream connection after passing exactly `limit` items
    pub fn take(self, limit: usize) -> PipeLine<In, Out> {
        panic!("TODO: Implement take limit check");
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
