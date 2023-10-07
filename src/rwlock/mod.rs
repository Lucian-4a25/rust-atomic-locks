use atomic_wait::{wait, wake_all, wake_one};
use std::{
    cell::UnsafeCell,
    fmt::Write,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicU32, AtomicUsize, Ordering},
};

pub struct RwLock<T> {
    // u32:MAX represents Write Locked
    // 0 represents UNLOCKED
    // others represents the number of READER LOCKS
    state: AtomicU32,
    /// Incremented to wake up writers.
    writer_wake_counter: AtomicU32, // New!
    value: UnsafeCell<T>,
    // record the num of writers
    num_writers: AtomicU32,
}

unsafe impl<T> Sync for RwLock<T> where T: Send + Sync {}

#[allow(dead_code)]
impl<T> RwLock<T> {
    pub fn new(value: T) -> Self {
        Self {
            state: AtomicU32::new(0),
            writer_wake_counter: AtomicU32::new(0),
            value: UnsafeCell::new(value),
            num_writers: AtomicU32::new(0),
        }
    }

    pub fn read(&self) -> ReadGuide<'_, T> {
        let mut n = self.state.load(Ordering::Relaxed);
        loop {
            if n % 2 == 0 {
                match self
                    .state
                    .compare_exchange(n, n + 2, Ordering::Acquire, Ordering::Relaxed)
                {
                    Ok(_) => return ReadGuide { mutex: self },
                    Err(p) => {
                        n = p;
                    }
                }
            }

            // block more readers if there are writers waiting
            if n % 2 == 1 {
                wait(&self.state, u32::MAX);
                n = self.state.load(Ordering::Relaxed);
            }
        }
    }

    pub fn write(&self) -> WriteGuide<'_, T> {
        let mut n = self.state.load(Ordering::Relaxed);
        self.num_writers.fetch_add(1, Ordering::Relaxed);

        loop {
            if n <= 1 {
                match self
                    .state
                    .compare_exchange(n, u32::MAX, Ordering::Acquire, Ordering::Relaxed)
                {
                    Ok(_) => return WriteGuide { mutex: self },
                    Err(e) => {
                        n = e;
                        continue;
                    }
                }
            }

            if n % 2 == 0 {
                match self
                    .state
                    .compare_exchange(n, n + 1, Ordering::Acquire, Ordering::Relaxed)
                {
                    Ok(_) => {}
                    Err(e) => {
                        n = e;
                        continue;
                    }
                }
            }

            // before the writer get into sleep, we must confirm it could not get the lock and make the state odd.
            let w = self.writer_wake_counter.load(Ordering::Relaxed);
            n = self.state.load(Ordering::Relaxed);
            if n >= 2 {
                wait(&self.writer_wake_counter, w);
                n = self.state.load(Ordering::Relaxed);
            }
        }
    }
}

pub struct WriteGuide<'a, T> {
    mutex: &'a RwLock<T>,
}

pub struct ReadGuide<'a, T> {
    mutex: &'a RwLock<T>,
}

impl<T> Drop for ReadGuide<'_, T> {
    fn drop(&mut self) {
        if self.mutex.state.fetch_sub(2, Ordering::Relaxed) == 3 {
            self.mutex
                .writer_wake_counter
                .fetch_add(1, Ordering::Release);
            wake_one(&self.mutex.writer_wake_counter);
        }
    }
}

impl<'a, T> Deref for ReadGuide<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.mutex.value.get() }
    }
}

impl<'a, T> Deref for WriteGuide<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.mutex.value.get() }
    }
}

impl<'a, T> DerefMut for WriteGuide<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.mutex.value.get() }
    }
}

impl<'a, T> Drop for WriteGuide<'a, T> {
    fn drop(&mut self) {
        let pre_num_writers = self.mutex.num_writers.fetch_sub(1, Ordering::Release);
        if pre_num_writers > 1 {
            self.mutex.state.store(1, Ordering::Release);
            self.mutex
                .writer_wake_counter
                .fetch_add(1, Ordering::Release);
            wake_one(&self.mutex.writer_wake_counter);
        } else {
            // it's alright if there is writer get into in this moment, cause it's negligible performance lost.
            self.mutex.state.store(0, Ordering::Release);
            wake_all(&self.mutex.state);
        }
    }
}

#[test]
fn test_rw_lock() {
    let rwlock = RwLock::new(0);
    std::thread::scope(|s| {
        for _ in 0..30 {
            s.spawn(|| {
                let rlock = rwlock.read();
                println!("read lock: {}", *rlock);
            });
            s.spawn(|| {
                let mut wlock = rwlock.write();
                *wlock += 1;
                println!("write lock: {}", *wlock);
            });
        }
    });

    println!("over");
}

#[test]
fn test_u32_odd_even() {
    println!("{}", u32::MAX % 2);
}
