use crate::Pipeline;

impl<In, Out> Pipeline<In, Out>
where
    In: Send + 'static,
    Out: Send + 'static,
{
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
            transform: Some(Box::new(move |source_rx, final_tx, tasks| {
                // 1. Create the junction channel between Pipeline A and Pipeline B
                let (mid_tx, mid_rx) = tokio::sync::mpsc::channel::<Out>(capacity);

                // 2. Solder Pipeline A to the junction
                if let Some(a) = transform_a {
                    a(source_rx, mid_tx, tasks);
                }

                // 3. Solder Pipeline B to the junction
                if let Some(b) = transform_b {
                    b(mid_rx, final_tx, tasks);
                }
            })),
        }
    }

    /// Zips another stream into this one, combining both inputs into a new output
    pub fn join_with<Other, NewOut, F>(
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
        let capacity = self.capacity;

        Pipeline {
            capacity,
            transform: Some(Box::new(move |source_rx, final_tx, tasks| {
                let (mid_tx, mut mid_rx) = tokio::sync::mpsc::channel::<Out>(capacity);

                if let Some(prev) = previous_transform {
                    prev(source_rx, mid_tx, tasks);
                }

                let handle = tokio::spawn(async move {
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
                tasks.push(handle);
            })),
        }
    }

    /// Clones the stream, sending one copy down the `branch` pipeline,
    /// and continuing the other copy down the main pipeline.
    ///
    /// **Note:** This acts as an 'eavesdropper'. The branch is strictly dependent on the
    /// main pipeline's lifecycle. If the main pipeline stops accepting data
    /// (e.g., via `.take()`), this operator stops pulling from upstream, effectively
    /// terminating the branch as well.
    pub fn tee<BranchOut>(self, mut branch: Pipeline<Out, BranchOut>) -> Pipeline<In, Out>
    where
        Out: Clone + Send + Sync + 'static,
        BranchOut: Send + 'static,
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

                // Prepare the branch
                let (branch_in_tx, branch_in_rx) = tokio::sync::mpsc::channel::<Out>(capacity);
                let (branch_out_tx, mut branch_out_rx) =
                    tokio::sync::mpsc::channel::<BranchOut>(capacity);

                if let Some(branch_compile) = branch.transform.take() {
                    branch_compile(branch_in_rx, branch_out_tx, tasks);
                }

                // Sink the branch output to nowhere
                let branch_exhaust_handle =
                    tokio::spawn(async move { while branch_out_rx.recv().await.is_some() {} });
                tasks.push(branch_exhaust_handle);

                let main_handle = tokio::spawn(async move {
                    while let Some(item) = mid_rx.recv().await {
                        // Send to main downstream FIRST
                        if final_tx.send(item.clone()).await.is_err() {
                            break;
                        }

                        // Send to branch only if main is still alive
                        let _ = branch_in_tx.send(item).await;
                    }
                });
                tasks.push(main_handle);
            })),
        }
    }

    /// Clones the stream to both a `branch` pipeline and the main pipeline.
    ///
    /// Unlike `tee`, `broadcast` will keep pulling from upstream as long as
    /// **at least one** of the output channels (main or branch) is still alive.
    pub fn broadcast<BranchOut>(self, mut branch: Pipeline<Out, BranchOut>) -> Pipeline<In, Out>
    where
        Out: Clone + Send + Sync + 'static,
        BranchOut: Send + 'static,
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

                // Prepare the branch
                let (branch_in_tx, branch_in_rx) = tokio::sync::mpsc::channel::<Out>(capacity);
                let (branch_out_tx, mut branch_out_rx) =
                    tokio::sync::mpsc::channel::<BranchOut>(capacity);

                if let Some(branch_compile) = branch.transform.take() {
                    branch_compile(branch_in_rx, branch_out_tx, tasks);
                }

                // Sink the branch output
                let branch_exhaust_handle =
                    tokio::spawn(async move { while branch_out_rx.recv().await.is_some() {} });
                tasks.push(branch_exhaust_handle);

                let main_handle = tokio::spawn(async move {
                    let mut main_alive = true;
                    let mut branch_alive = true;

                    while let Some(item) = mid_rx.recv().await {
                        if branch_alive {
                            if branch_in_tx.send(item.clone()).await.is_err() {
                                branch_alive = false;
                            }
                        }

                        if main_alive {
                            if final_tx.send(item).await.is_err() {
                                main_alive = false;
                            }
                        }

                        // Stop pulling from upstream only if both downstreams are dead
                        if !main_alive && !branch_alive {
                            break;
                        }
                    }
                });
                tasks.push(main_handle);
            })),
        }
    }

    /// Zips another stream into this one, returning a tuple of (SelfItem, OtherItem)
    pub fn zip<Other>(
        self,
        mut other_rx: tokio::sync::mpsc::Receiver<Other>,
    ) -> Pipeline<In, (Out, Other)>
    where
        Other: Send + 'static,
        Out: Send + 'static,
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
                    while let Some(item_self) = mid_rx.recv().await {
                        if let Some(item_other) = other_rx.recv().await {
                            if final_tx.send((item_self, item_other)).await.is_err() {
                                return;
                            }
                        } else {
                            return;
                        }
                    }
                });
                tasks.push(handle);
            })),
        }
    }

    /// Merges another stream of the same type into this one.
    pub fn merge(self, mut other_rx: tokio::sync::mpsc::Receiver<Out>) -> Pipeline<In, Out>
    where
        Out: Send + 'static,
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

                let final_tx_clone = final_tx.clone();
                let handle_a = tokio::spawn(async move {
                    while let Some(item) = mid_rx.recv().await {
                        if final_tx_clone.send(item).await.is_err() {
                            break;
                        }
                    }
                });
                tasks.push(handle_a);

                let handle_b = tokio::spawn(async move {
                    while let Some(item) = other_rx.recv().await {
                        if final_tx.send(item).await.is_err() {
                            break;
                        }
                    }
                });
                tasks.push(handle_b);
            })),
        }
    }
}
