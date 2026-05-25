use downstream_rs::Pipeline;
use futures::stream::{self};

#[tokio::main]
async fn main() {
    let fib_stream = stream::unfold((0u128, 1u128), |(a, b)| async move {
        let next = a + b;
        Some((a, (b, next)))
    });

    // 1. Setup a branch that wants 5 items
    let branch_pipeline = Pipeline::start().take(10).sink(|num: u128| {
        println!("[BRANCH] Received: {}", num);
    });

    // 2. Run with broadcast (independent)
    // Even though main only takes 2, the branch should get all 5.
    // Now we DON'T need any manual sleeps!
    println!("--- Testing BROADCAST (Any-driven) ---");
    Pipeline::with_capacity(1)
        .broadcast(branch_pipeline)
        .take(2)
        .sink(|num| {
            println!("[MAIN] Received: {}", num);
        })
        .run(fib_stream)
        .await
        .unwrap();

    println!("Broadcast test finished (Automated completion).\n");

    // 3. Reset stream for tee test
    let fib_stream_2 = stream::unfold((0u128, 1u128), |(a, b)| async move {
        let next = a + b;
        Some((a, (b, next)))
    });

    let branch_pipeline_2 = Pipeline::start().take(5).sink(|num: u128| {
        println!("[BRANCH] Received: {}", num);
    });

    // 4. Run with tee (Main-driven)
    println!("--- Testing TEE (Main-driven) ---");
    Pipeline::with_capacity(1)
        .tee(branch_pipeline_2)
        .take(2)
        .sink(|num| {
            println!("[MAIN] Received: {}", num);
        })
        .run(fib_stream_2)
        .await
        .unwrap();

    println!("Tee test finished (Automated completion).");
}
