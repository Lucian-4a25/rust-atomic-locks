use std::{
    sync::{
        atomic::{AtomicU16, Ordering},
        Arc,
    },
    thread,
};

#[allow(dead_code)]
pub fn happen_beofore() {
    thread::scope(|s| {
        for _ in 0..500000 {
            let x = Arc::new(AtomicU16::new(0));
            let y = Arc::new(AtomicU16::new(0));
            let x2 = x.clone();
            let y2 = y.clone();
            s.spawn(move || {
                x2.store(10, Ordering::Relaxed);
                y2.store(20, Ordering::Relaxed);
            });
            s.spawn(move || {
                let y = y.load(Ordering::Relaxed);
                let x = x.load(Ordering::Relaxed);

                if x == 0 {
                    assert_eq!(y, 0);
                } else if y == 20 {
                    assert_eq!(x, 10);
                }
            });
        }
    });
}
