# Downstream-RS

A push-based asynchronous data orchestration library for building complex pipeline topologies.

## Quick Start

```rust
use downstream_rs::pipeline;

#[tokio::main]
async fn main() {
    let items = futures::stream::iter(0..10);

    // Declare the pipeline
    let pipeline = pipeline![
        pipe(|x: i32| Some(x)),
        sink(|x| print!("{} ", x))
    ];

    // Run the pipeline individually
    pipeline.run(items).await.unwrap();
    
    // Output: 0 1 2 3 4 5 6 7 8 9 
}
```

## Examples

### Audio Signal Smoothing
Use `chunk` to batch spiky raw samples and calculate a moving average to smooth the signal.

```rust
// A stream of spiky audio data from an external source
let raw_samples = get_audio_stream();

pipeline![
    // Group into batches of 10 to capture local variance
    chunk(10),

    // Smooth the signal by averaging the batch
    pipe(|batch: Vec<f32>| {
        let sum: f32 = batch.iter().sum();
        Some(sum / batch.len() as f32)
    }),

    sink(|avg| print!("{:.2} ", avg))
]
.run(raw_samples)
.await
.unwrap();
```

### Complex Topology
Build non-linear flows using primitives like `broadcast`.

```rust
pipeline![
    broadcast(pipeline![
        sink(|avg| eprintln!("[LOG] Current Volume: {:.2}", avg))
    ]),
    sink(|avg| play_audio(avg))
]
```

## Features

- **Push-based Architecture:** Data is pushed through the pipeline as it arrives, making it highly responsive for event-driven systems.
- **Topology Primitives:** Easily build non-linear flows with `broadcast`, `tee`, `route`, and `merge`.
- **Automatic Parallelism:** Scale compute-heavy stages across all CPU cores with a single `.par_pipe()` call.
- **Reliable Lifecycle:** Automatic tracking and awaiting of all background tasks and branches.
- **Ergonomic Macro:** Define entire pipelines using the `pipeline!` declarative syntax.

## Performance Note

Downstream-RS is a **system-level orchestrator**. It is optimized for handling I/O-bound or compute-heavy tasks. Because it uses async channels and task switching between every stage, it is not intended for high-frequency, low-latency instruction-level math, where a raw `while` loop would be significantly faster.

## Operators

- `pipe`: Inline transformation and filtering.
- `par_pipe`: Multi-worker parallel processing (auto-scales to CPU cores).
- `route`: Type-safe conditional diversion to a branch.
- `broadcast`: Unconditional independent branching.
- `tee`: Unconditional dependent branching (eavesdropping).
- `zip` / `merge`: Combining streams.
- `take` / `wait` / `window` / `chunk`: Flow control and batching.

## Why Downstream?

Standard Rust streams are excellent for linear data processing. However, as data flows become more complex—involving branching, routing, or parallel execution—the implementation often requires significant boilerplate for manual channel management and task lifecycle tracking. 

Downstream-RS provides a declarative abstraction for these operations, handling the underlying task orchestration and backpressure so you can focus on the pipeline logic.

## License

MIT or Apache-2.0
