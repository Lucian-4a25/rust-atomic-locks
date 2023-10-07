use crate::mutex::{Mutex, MutexGuard};
use atomic_wait::{wait, wake_all, wake_one};
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use std::thread;

pub struct CondVar {
    counter: AtomicU32,
    num_waiters: AtomicUsize,
}

#[allow(dead_code)]
impl CondVar {
    pub fn new() -> Self {
        Self {
            counter: AtomicU32::new(0),
            // 使用 num_waiters 来避免没有必要的
            num_waiters: AtomicUsize::new(0),
        }
    }

    fn wait<'a, T>(&self, v: MutexGuard<'a, T>) -> MutexGuard<'a, T> {
        self.num_waiters.fetch_add(1, Ordering::Release);
        let counter = self.counter.load(Ordering::Relaxed);
        let mutex = v.mutex;

        drop(v);

        // TODO: avoid spurious wake.
        loop {
            wait(&self.counter, counter);
            // self implemention to avoid spurious wake.
            if self.counter.load(Ordering::Relaxed) != counter {
                break;
            }
        }

        self.num_waiters.fetch_sub(1, Ordering::Release);
        mutex.lock()
    }

    #[allow(dead_code)]
    fn notify_one(&self) {
        if self.num_waiters.load(Ordering::Acquire) > 0 {
            self.counter.fetch_add(1, Ordering::Relaxed);
            wake_one(&self.counter);
        }
    }

    #[allow(dead_code)]
    fn notify_all(&self) {
        if self.num_waiters.load(Ordering::Acquire) > 0 {
            self.counter.fetch_add(1, Ordering::Relaxed);
            wake_all(&self.counter);
        }
    }
}

#[test]
fn test_condvar() {
    let m = Mutex::new(0);
    let cond_v = CondVar::new();

    thread::scope(|s| {
        s.spawn(|| {
            for _ in 0..1000 {
                *m.lock() += 1;
                cond_v.notify_one();
            }
        });

        let mut m_guide = m.lock();
        while *m_guide != 1000 {
            m_guide = cond_v.wait(m_guide);
        }
    });

    assert_eq!(1000, *m.lock());
}
