use std::{
    cell::UnsafeCell,
    hint::spin_loop,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicBool, Ordering},
    thread,
};

pub struct SpinLock<T> {
    locked: AtomicBool,
    value: UnsafeCell<T>,
}

impl<T> SpinLock<T> {
    #[allow(dead_code)]
    pub fn new(val: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            value: UnsafeCell::new(val),
        }
    }

    #[allow(dead_code)]
    pub fn lock(&self) -> Guard<'_, T> {
        while self.locked.swap(true, Ordering::Acquire) {
            spin_loop();
        }
        // unsafe { &mut *self.value.get() }
        Guard { lock: self }
    }

    #[allow(dead_code)]
    /// Safety: The &mut T from lock() must be gone!
    /// (And no cheating by keeping reference to fields of that T around!)
    pub unsafe fn unlock(&self) {
        self.locked.store(false, Ordering::Relaxed);
    }
}

unsafe impl<T> Sync for SpinLock<T> where T: Send {}

pub struct Guard<'a, T> {
    lock: &'a SpinLock<T>,
}

impl<'a, T> Deref for Guard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.value.get() }
    }
}

impl<'a, T> DerefMut for Guard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.value.get() }
    }
}

impl<'a, T> Drop for Guard<'a, T> {
    fn drop(&mut self) {
        self.lock.locked.store(false, Ordering::Release);
    }
}

#[allow(dead_code)]
pub fn spin_lock_usage() {
    let x = SpinLock::new(Vec::new());
    thread::scope(|s| {
        s.spawn(|| {
            x.lock().push(1);
        });

        s.spawn(|| {
            let mut l = x.lock();
            l.push(2);
            l.push(2);
        });
    });

    let g = x.lock();

    assert!(g.as_slice() == [1, 2, 2] || g.as_slice() == [2, 2, 1]);
    println!("the result of spin lock is: {:?}", g.as_slice());
}
