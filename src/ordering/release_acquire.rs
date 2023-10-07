use std::{
    sync::{
        atomic::{AtomicBool, AtomicI16, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

#[allow(dead_code)]
pub fn release_acquire_ordering() {
    let t1: thread::JoinHandle<()> = thread::spawn(|| {
        for _ in 0..10000 {
            let data = Arc::new(AtomicI16::new(0));
            let ddata = Arc::new(AtomicI16::new(0));
            let dddata = Arc::new(AtomicI16::new(0));
            let ready = Arc::new(AtomicBool::new(false));
            let data2 = data.clone();
            let ddata2 = ddata.clone();
            let dddata2 = dddata.clone();
            let ready2 = ready.clone();

            thread::spawn(move || {
                data2.store(-123, Ordering::Relaxed);
                ddata2.store(-111, Ordering::Relaxed);
                dddata2.store(-11, Ordering::Relaxed);
                // Release 表示其前所有的原子操作都在读取其对应的 thread 的 Acquire 前发生
                ready2.store(true, Ordering::Release);
            });

            // Acquire 表示读取的原子值之后的操作，都发生在对应的 Release 后
            while !ready.load(Ordering::Acquire) {
                // println!("waiting....");
                thread::sleep(Duration::from_millis(100));
            }

            assert_eq!(-123, data.load(Ordering::Relaxed));
            assert_eq!(-111, ddata.load(Ordering::Relaxed));
            assert_eq!(-11, dddata.load(Ordering::Relaxed));
        }
    });

    t1.join().unwrap();
}
