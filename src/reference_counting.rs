use std::{sync::Arc, thread};

#[allow(dead_code)]
pub fn reference_counting() {
    // atomic reference counting
    let a = Arc::new([1, 2, 3]);
    // let b = a.clone();

    let t1 = thread::spawn({
        let a = a.clone();
        move || {
            // a.sort();
            dbg!(a);
        }
    });

    let t2 = thread::spawn({
        let a = a.clone();
        move || {
            dbg!(a);
        }
    });

    dbg!(a);

    t1.join().unwrap();
    t2.join().unwrap();
}
