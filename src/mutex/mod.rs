use atomic_wait::{wait, wake_all, wake_one};
use std::{
    cell::UnsafeCell,
    hint::spin_loop,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicI16, AtomicU16, AtomicU32, Ordering},
    thread,
    time::Instant,
};

const UNLOCKED: u32 = 0;
const LOCKED: u32 = 1;
const LOCKED_WITH_WAITERS: u32 = 2;

pub struct Mutex<T> {
    state: AtomicU32,
    value: UnsafeCell<T>,
}

unsafe impl<T> Sync for Mutex<T> where T: Send {}

pub struct MutexGuard<'a, T> {
    pub mutex: &'a Mutex<T>,
}

impl<'a, T> Deref for MutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.mutex.value.get() }
    }
}

impl<T> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.mutex.value.get() }
    }
}

#[allow(dead_code)]
fn lock_contended_v1(state: &AtomicU32) {
    loop {
        wait(state, LOCKED);
        if state
            .compare_exchange(UNLOCKED, LOCKED, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
        {
            break;
        }
    }
}

#[allow(dead_code)]
fn lock_contended_v2(state: &AtomicU32) {
    // still locked, the thread need to wait
    while state.swap(LOCKED_WITH_WAITERS, Ordering::Release) != UNLOCKED {
        wait(state, LOCKED_WITH_WAITERS);
    }
}

#[allow(dead_code)]
fn lock_contended(state: &AtomicU32) {
    let mut spin_count = 0;

    while state.load(Ordering::Relaxed) == LOCKED && spin_count < 100 {
        spin_loop();
        spin_count += 1;
    }

    if state
        .compare_exchange(UNLOCKED, LOCKED, Ordering::Acquire, Ordering::Relaxed)
        .is_ok()
    {
        return;
    }

    // still locked, the thread need to wait
    while state.swap(LOCKED_WITH_WAITERS, Ordering::Release) != 0 {
        wait(state, LOCKED_WITH_WAITERS);
    }
}

impl<T> Mutex<T> {
    #[allow(dead_code)]
    pub fn new(v: T) -> Self {
        Self {
            state: AtomicU32::new(0),
            value: UnsafeCell::new(v),
        }
    }

    #[allow(dead_code)]
    pub fn lock(&self) -> MutexGuard<'_, T> {
        if self
            .state
            .compare_exchange(UNLOCKED, LOCKED, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            lock_contended_v1(&self.state);
            // lock_contended_v2(&self.state);
            // lock_contended(&self.state);
        }

        MutexGuard { mutex: self }
    }
}

impl<T> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        // sometimes we don't need to wake thread, so we have to make sure there is thread to wake up
        // if self.mutex.state.swap(UNLOCKED, Ordering::Release) == LOCKED_WITH_WAITERS {
        //     wake_one(&self.mutex.state);
        // }

        // for v1
        self.mutex.state.store(UNLOCKED, Ordering::Release);
        wake_one(&self.mutex.state);
    }
}

#[test]
fn test_uncondended() {
    let m = Mutex::new(0);
    std::hint::black_box(&m);
    let start = Instant::now();
    for _ in 0..5_000_000 {
        *m.lock() += 1;
    }
    let duration = start.elapsed();
    println!("locked {} times in {:?}", *m.lock(), duration);
}

#[test]
fn test_condeneded() {
    let m = Mutex::new(0);
    std::hint::black_box(&m);
    let start = Instant::now();
    thread::scope(|s| {
        for _ in 0..10 {
            s.spawn(|| {
                for _ in 0..5_000_000 {
                    *m.lock() += 1;
                }
            });
        }
    });
    let duration = start.elapsed();
    println!("locked {} times in {:?}", *m.lock(), duration);
}

// 原子值操作溢出之后会从最小值开始计数
#[test]
fn test_atomic_overflow() {
    let a = AtomicI16::new(i16::MAX);
    let v1 = a.fetch_add(1, Ordering::Relaxed);
    let v2 = a.fetch_add(1, Ordering::Relaxed);
    let v3 = a.fetch_add(1, Ordering::Relaxed);
    println!("{v1}, {v2}, {v3}");
}
