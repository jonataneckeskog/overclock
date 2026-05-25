use downstream_rs::Pipeline;
use futures::stream::{self};

#[tokio::main]
async fn main() {
    let fib_stream = stream::unfold((0u64, 1u64), |(a, b)| async move {
        let next = a + b;

        Some((a, (b, next)))
    });

    // Define and run the pipeline
    Pipeline::with_capacity(32)
        .take(10)
        .sink(|num| println!("Fibonacci: {}", num))
        .run(fib_stream)
        .await
        .unwrap();
}
