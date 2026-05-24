use tokio::sync::mpsc;

pub struct PipeLine<In, Out> {
    pub transform:
        Option<Box<dyn FnOnce(mpsc::Receiver<In>, mpsc::Sender<Out>) + Send + Sync + 'static>>,
}

impl PipeLine<(), ()> {
    pub fn new() -> Self {
        panic!("TODO: Initialize an empty PipeLine blueprint");
    }
}

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
        F: Fn(Out) -> NewOut + Send + Sync + 'static,
        NewOut: Send + 'static,
    {
        panic!("TODO: Implement map/transform logic");
    }

    /// Executes a side effect (like saving to a DB) and passes the data through untouched
    pub fn tap<F>(self, action: F) -> PipeLine<In, Out>
    where
        F: Fn(&Out) + Send + Sync + 'static, // Passing by reference is usually best for taps!
    {
        panic!("TODO: Implement tap/side-effect logic");
    }

    /// Closes the downstream connection after passing exactly `limit` items
    pub fn take(self, limit: usize) -> PipeLine<In, Out> {
        panic!("TODO: Implement take limit check");
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

    /// The terminal node: consumes the blueprint, wires all channels, and fires up the Tokio tasks
    pub fn sink<F>(self, action: F)
    where
        F: Fn(Out) + Send + Sync + 'static,
    {
        panic!("TODO: Implement sink, wire channels, and start execution");
    }
}
