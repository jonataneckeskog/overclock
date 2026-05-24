use tokio::sync::mpsc;

use crate::PipeLine;

impl<In, Out> PipeLine<In, Out>
where
    In: Send + 'static,
    Out: Send + 'static,
{
    /// Attaches a real data source and defines the starting type for the network
    pub fn stream<NewIn>(self, rx: mpsc::Receiver<NewIn>) -> PipeLine<NewIn, NewIn>
    where
        NewIn: Send + 'static,
    {
        panic!("TODO: Implement stream source injection");
    }

    /// Transforms the data from type `Out` to type `NewOut`
    pub fn pipe<F, NewOut>(self, transform: F) -> PipeLine<In, NewOut>
    where
        // The closure MUST return an Option!
        F: Fn(Out) -> Option<NewOut> + Send + Sync + 'static,
        NewOut: Send + 'static,
    {
        let previous_transform = self.transform;

        PipeLine {
            transform: Some(Box::new(move |source_rx, final_tx| {
                let (mid_tx, mut mid_rx) = tokio::sync::mpsc::channel::<Out>(32);

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

    /// The terminal node: consumes the blueprint, wires all channels, and fires up the Tokio tasks
    pub fn sink<F>(self, action: F)
    where
        F: Fn(Out) + Send + Sync + 'static,
    {
        panic!("TODO: Implement sink, wire channels, and start execution");
    }
}
