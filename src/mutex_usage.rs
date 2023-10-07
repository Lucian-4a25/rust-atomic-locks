use std::{
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

#[allow(dead_code)]
pub fn mutex_usage() {
    let instant = Instant::now();
    let n: Arc<Mutex<i32>> = Arc::new(Mutex::new(0));

    thread::scope(|s| {
        for _ in 0..10 {
            let n = n.clone();
            s.spawn(move || {
                let mut guide = n.lock().unwrap();
                *guide += 1;
                drop(guide);
                thread::sleep(Duration::from_secs(1))
            });
        }
    });

    println!("the execution time is {:?}", instant.elapsed());
    assert_eq!(Arc::into_inner(n).unwrap().into_inner().unwrap(), 10);
}
