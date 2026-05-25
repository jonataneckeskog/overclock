use downstream_rs::Pipeline;
use futures::stream;
use std::time::Instant;
use std::hint::black_box;

#[tokio::main]
async fn main() {
    let count = 1_000_000;
    let items = || stream::iter(0..count);

    println!("--- Real-World Comparison (with black_box): 1,000,000 items ---");

    // 1. Raw While Loop (Optimizations Disabled)
    let start_loop = Instant::now();
    let mut i = 0;
    while i < count {
        if black_box(i) % 100000 == 0 {
            black_box(i);
        }
        i += 1;
    }
    let loop_duration = start_loop.elapsed();
    println!("Raw While Loop: {:?}", loop_duration);

    // 2. Downstream Pipeline
    let start_pipe = Instant::now();
    Pipeline::start()
        .pipe(|x| if black_box(x) % 100000 == 0 { Some(black_box(x)) } else { None })
        .sink(|x| { black_box(x); }) 
        .run(items())
        .await
        .unwrap();
    let pipe_duration = start_pipe.elapsed();
    println!("Downstream Pipe: {:?}", pipe_duration);

    println!("\nRatio (Pipe / Loop): {:.2}x slower", pipe_duration.as_secs_f64() / loop_duration.as_secs_f64());
}
