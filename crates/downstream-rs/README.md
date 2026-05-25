# Downstream-RS

A push-based asynchronous data orchestration library for Rust.

Downstream-RS provides a declarative DSL for building complex data processing topologies (graphs) using Tokio tasks. It simplifies branching, merging, and parallel processing of async streams.

## Features

- **Push-based Architecture:** Data is pushed through the pipeline as it arrives, making it highly responsive for event-driven systems.
- **Topology Primitives:** Easily build non-linear flows with `broadcast`, `tee`, `route`, and `merge`.
- **Automatic Parallelism:** Scale compute-heavy stages across all CPU cores with a single `.par_pipe()` call.
- **Reliable Lifecycle:** Automatic tracking and awaiting of all background tasks and branches.
- **Ergonomic Macro:** Define entire pipelines using the `pipeline!` declarative syntax.

## Why Downstream?

Standard Rust `Streams` are excellent for linear, pull-based data. However, as soon as you need to:
1. Send the same data to three different places simultaneously.
2. Route items to different handlers based on their type.
3. Automatically parallelize a single step of a pipe.

...the boilerplate of manual `mpsc` channels, `Arc<Mutex>` locks, and `tokio::spawn` management becomes overwhelming. Downstream-RS abstracts this "plumbing" into a clean, readable API.

## Performance Note

Downstream-RS is a **system-level orchestrator**. It is optimized for handling I/O-bound or compute-heavy tasks. Because it uses async channels and task switching between every stage, it is not intended for high-frequency, low-latency instruction-level math (like summing 10 million integers), where a raw `while` loop would be significantly faster.

## Quick Start

```rust
use downstream_rs::pipeline;

#[tokio::main]
async fn main() {
    let items = futures::stream::iter(0..100);

    pipeline![
        // 1. Double the numbers
        pipe(|x: i32| Some(x * 2)),
        
        // 2. Parallel heavy processing (unordered)
        par_pipe(|x| {
            // simulate work
            Some(x)
        }),
        
        // 3. Final action
        sink(|x| println!("Result: {}", x))
    ]
    .run(items)
    .await
    .unwrap();
}
```

## Operators

- `pipe`: Inline transformation and filtering.
- `par_pipe`: Multi-worker parallel processing (auto-scales to CPU cores).
- `route`: Type-safe conditional diversion to a branch.
- `broadcast`: Unconditional independent branching.
- `tee`: Unconditional dependent branching (eavesdropping).
- `zip` / `merge`: Combining streams.
- `take` / `wait` / `window` / `chunk`: Flow control and batching.

## License

MIT or Apache-2.0
