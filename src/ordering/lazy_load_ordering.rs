use std::{
    ptr::null_mut,
    sync::atomic::{AtomicPtr, Ordering},
};

#[allow(dead_code)]
pub fn get_init() -> usize {
    static PTR: AtomicPtr<usize> = AtomicPtr::new(null_mut());

    let mut p = PTR.load(Ordering::Acquire);

    if p.is_null() {
        p = Box::into_raw(Box::new(32));

        if let Err(e) = PTR.compare_exchange(null_mut(), p, Ordering::Release, Ordering::Acquire) {
            drop(p);
            p = e;
        }
    }

    unsafe { *p }
}
