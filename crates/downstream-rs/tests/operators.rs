use downstream_rs::Pipeline;
use futures::stream;
use std::sync::{Arc, Mutex};

#[tokio::test]
async fn test_pipe_transformation() {
    let items = stream::iter(1..=3);
    let results = Arc::new(Mutex::new(Vec::new()));
    let results_clone = results.clone();

    Pipeline::start()
        .pipe(|x: i32| Some(x * 2))
        .sink(move |x| {
            results_clone.lock().unwrap().push(x);
        })
        .run(items)
        .await
        .unwrap();

    let final_results = results.lock().unwrap();
    assert_eq!(*final_results, vec![2, 4, 6]);
}

#[tokio::test]
async fn test_pipe_filtering() {
    let items = stream::iter(1..=5);
    let results = Arc::new(Mutex::new(Vec::new()));
    let results_clone = results.clone();

    Pipeline::start()
        .pipe(|x: i32| if x % 2 == 0 { Some(x) } else { None })
        .sink(move |x| {
            results_clone.lock().unwrap().push(x);
        })
        .run(items)
        .await
        .unwrap();

    let final_results = results.lock().unwrap();
    assert_eq!(*final_results, vec![2, 4]);
}

#[tokio::test]
async fn test_par_pipe() {
    let items = stream::iter(1..=10);
    let results = Arc::new(Mutex::new(Vec::new()));
    let results_clone = results.clone();

    Pipeline::start()
        .par_pipe(|x: i32| Some(x * 10))
        .sink(move |x| {
            results_clone.lock().unwrap().push(x);
        })
        .run(items)
        .await
        .unwrap();

    let mut final_results = results.lock().unwrap().clone();
    final_results.sort(); // Sorting because par_pipe is unordered
    let expected: Vec<i32> = (1..=10).map(|x| x * 10).collect();
    assert_eq!(final_results, expected);
}

#[tokio::test]
async fn test_tee() {
    let items = stream::iter(1..=3);
    let main_results = Arc::new(Mutex::new(Vec::new()));
    let branch_results = Arc::new(Mutex::new(Vec::new()));
    
    let main_clone = main_results.clone();
    let branch_clone = branch_results.clone();

    let branch = Pipeline::start().sink(move |x| {
        branch_clone.lock().unwrap().push(x);
    });

    Pipeline::start()
        .tee(branch)
        .sink(move |x| {
            main_clone.lock().unwrap().push(x);
        })
        .run(items)
        .await
        .unwrap();

    assert_eq!(*main_results.lock().unwrap(), vec![1, 2, 3]);
    assert_eq!(*branch_results.lock().unwrap(), vec![1, 2, 3]);
}

#[tokio::test]
async fn test_broadcast() {
    let items = stream::iter(1..=3);
    let main_results = Arc::new(Mutex::new(Vec::new()));
    let branch_results = Arc::new(Mutex::new(Vec::new()));
    
    let main_clone = main_results.clone();
    let branch_clone = branch_results.clone();

    let branch = Pipeline::start().sink(move |x| {
        branch_clone.lock().unwrap().push(x);
    });

    Pipeline::start()
        .broadcast(branch)
        .sink(move |x| {
            main_clone.lock().unwrap().push(x);
        })
        .run(items)
        .await
        .unwrap();

    assert_eq!(*main_results.lock().unwrap(), vec![1, 2, 3]);
    assert_eq!(*branch_results.lock().unwrap(), vec![1, 2, 3]);
}

#[tokio::test]
async fn test_route() {
    let items = stream::iter(1..=4);
    let odd_results = Arc::new(Mutex::new(Vec::new()));
    let even_results = Arc::new(Mutex::new(Vec::new()));
    
    let odd_clone = odd_results.clone();
    let even_clone = even_results.clone();

    let even_branch = Pipeline::start().sink(move |x| {
        even_clone.lock().unwrap().push(x);
    });

    Pipeline::start()
        .route(even_branch, |x| {
            if x % 2 == 0 { Err(x) } else { Ok(x) }
        })
        .sink(move |x| {
            odd_clone.lock().unwrap().push(x);
        })
        .run(items)
        .await
        .unwrap();

    assert_eq!(*odd_results.lock().unwrap(), vec![1, 3]);
    assert_eq!(*even_results.lock().unwrap(), vec![2, 4]);
}

#[tokio::test]
async fn test_zip() {
    let items_main = stream::iter(1..=3);
    let (tx_other, rx_other) = tokio::sync::mpsc::channel(32);
    
    tokio::spawn(async move {
        for s in vec!["a", "b", "c"] {
            tx_other.send(s).await.unwrap();
        }
    });

    let results = Arc::new(Mutex::new(Vec::new()));
    let results_clone = results.clone();

    Pipeline::start()
        .zip(rx_other)
        .sink(move |(n, s)| {
            results_clone.lock().unwrap().push((n, s));
        })
        .run(items_main)
        .await
        .unwrap();

    assert_eq!(*results.lock().unwrap(), vec![(1, "a"), (2, "b"), (3, "c")]);
}

#[tokio::test]
async fn test_merge() {
    let items_main = stream::iter(vec![1, 2]);
    let (tx_other, rx_other) = tokio::sync::mpsc::channel(32);
    
    tokio::spawn(async move {
        tx_other.send(3).await.unwrap();
        tx_other.send(4).await.unwrap();
    });

    let results = Arc::new(Mutex::new(Vec::new()));
    let results_clone = results.clone();

    Pipeline::start()
        .merge(rx_other)
        .sink(move |x| {
            results_clone.lock().unwrap().push(x);
        })
        .run(items_main)
        .await
        .unwrap();

    let mut final_results = results.lock().unwrap().clone();
    final_results.sort();
    assert_eq!(final_results, vec![1, 2, 3, 4]);
}

#[tokio::test]
async fn test_apply() {
    let items = stream::iter(1..=2);
    let results = Arc::new(Mutex::new(Vec::new()));
    let results_clone = results.clone();

    let other_pipeline = Pipeline::start().sink(move |x: i32| {
        results_clone.lock().unwrap().push(x);
    });

    Pipeline::start()
        .apply(other_pipeline)
        .run(items)
        .await
        .unwrap();

    assert_eq!(*results.lock().unwrap(), vec![1, 2]);
}
