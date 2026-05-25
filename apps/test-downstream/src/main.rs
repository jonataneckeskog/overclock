use downstream_rs::Pipeline;
use futures::stream::{self};
use std::time::Instant;

#[tokio::main]
async fn main() {
    let items = || stream::iter(0..10);

    println!("--- Sequential vs Parallel Performance Test (10 items, 100ms delay each) ---");

    // 1. Sequential Test
    let start = Instant::now();
    Pipeline::start()
        .pipe(|x| Some(x))
        .sink(|_| {}) // Must terminate the pipeline
        .run(items())
        .await
        .unwrap();
    println!("Sequential took: {:?}", start.elapsed());

    // 2. Parallel Test
    let start = Instant::now();
    Pipeline::start()
        .par_pipe(|x| Some(x))
        .sink(|x| {
            print!("{} ", x); // Show that order is scrambled
        })
        .run(items())
        .await
        .unwrap();
    println!("\nParallel took: {:?}", start.elapsed());
}
