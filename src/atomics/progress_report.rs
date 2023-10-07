use std::{
    sync::atomic::{AtomicU64, AtomicUsize, Ordering},
    thread,
    time::{Duration, Instant},
};

#[allow(dead_code)]
pub fn display_update_progress() {
    let current = AtomicUsize::new(0);
    let total_time = AtomicU64::new(0);
    let max_time = AtomicU64::new(0);

    thread::scope(|s| {
        for _ in 0..5 {
            s.spawn(|| {
                for _ in 0..20 {
                    let start = Instant::now();
                    thread::sleep(Duration::from_millis(100));
                    let time_taken = start.elapsed().as_micros() as u64;
                    current.fetch_add(1, Ordering::Relaxed);
                    total_time.fetch_add(time_taken, Ordering::Relaxed);
                    max_time.fetch_max(time_taken, Ordering::Relaxed);
                }
            });
        }

        loop {
            let n = current.load(Ordering::Relaxed);
            let total_time = total_time.load(Ordering::Relaxed);
            let max_time = max_time.load(Ordering::Relaxed);

            if n == 100 {
                println!("Work completed!");
                break;
            }
            if n == 0 {
                println!("Working...");
            } else {
                println!(
                    "Work...{n}/100 done, {:?} average, {:?} peak",
                    total_time / n as u64,
                    max_time,
                );
            }

            thread::sleep(Duration::from_millis(200));
        }
    });
}
