use downstream_rs::Pipeline;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let (tx, rx) = mpsc::channel::<u64>(32);

    tokio::spawn(async move {
        let mut a = 0;
        let mut b = 1;

        loop {
            if tx.send(a).await.is_err() {
                println!("Generator shutting down.");
                break;
            }

            let next = a + b;
            a = b;
            b = next;
        }
    });

    // Define and run the pipeline
    Pipeline::with_capacity(32)
        .take(10)
        .sink(|num| println!("Fibonacci: {}", num))
        .run(rx)
        .await
        .unwrap();
}
