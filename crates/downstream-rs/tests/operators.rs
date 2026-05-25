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
