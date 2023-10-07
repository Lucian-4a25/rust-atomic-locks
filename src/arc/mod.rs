use std::{
    cell::UnsafeCell,
    hint::spin_loop,
    mem::ManuallyDrop,
    ops::Deref,
    ptr::NonNull,
    sync::atomic::{fence, AtomicUsize, Ordering},
    thread,
};

struct ArcData<T> {
    // Number of Arc's
    data_ref_count: AtomicUsize,
    // Number of `Weak`s, plus one if there are any `Arc`s.
    alloc_ref_count: AtomicUsize,
    /// The data. `None` if there's only weak pointers left.
    data: UnsafeCell<ManuallyDrop<T>>,
}

pub struct Arc<T> {
    ptr: NonNull<ArcData<T>>,
}

unsafe impl<T> Sync for Arc<T> where T: Sync + Send {}
unsafe impl<T> Send for Arc<T> where T: Send + Sync {}

impl<T> Arc<T> {
    #[allow(dead_code)]
    pub fn new(v: T) -> Self {
        Self {
            ptr: NonNull::from(Box::leak(Box::new(ArcData {
                data: UnsafeCell::new(ManuallyDrop::new(v)),
                data_ref_count: AtomicUsize::new(1),
                alloc_ref_count: AtomicUsize::new(1),
            }))),
        }
    }

    fn data(&self) -> &ArcData<T> {
        unsafe { self.ptr.as_ref() }
    }

    #[allow(dead_code)]
    pub fn downgrade(arc: &Self) -> Weak<T> {
        let mut n = arc.data().alloc_ref_count.load(Ordering::Relaxed);

        loop {
            if n == usize::MAX {
                spin_loop();
                n = arc.data().alloc_ref_count.load(Ordering::Relaxed);
                continue;
            }

            if let Err(e) = arc.data().alloc_ref_count.compare_exchange_weak(
                n,
                n + 1,
                Ordering::Acquire,
                Ordering::Relaxed,
            ) {
                n = e;
                continue;
            }

            return Weak { ptr: arc.ptr };
        }
    }

    // 之前的版本中，两个变量分别对应着 Arc 和 Weak 的数目，Arc Drop 对应的 Weak 也 Drop，
    // 那时只需要检查 alloc_ref_count 即可
    // 但现在所有的 Arc 都指向一个 Weak 计数，必须检查是否两个都为 1 来确保全局只有一个 Arc 可读引用。
    // 不管我们先检查哪一个，先检查的那个都有可能会发生变化，为了解决这个问题，将先检查的值临时改为 usize:MAX，临时阻塞相关值增加的
    // 具体情况可见例子中的循环存在的 Weak 和 Arc
    #[allow(dead_code)]
    pub fn get_mut(arc: &mut Self) -> Option<&mut T> {
        // 锁住 alloc_ref_count，避免在 get_mut 的期间进行 downgrade
        if arc
            .data()
            .alloc_ref_count
            .compare_exchange(1, usize::MAX, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            return None;
        }

        // 因为 alloc_ref_count 被暂时阻塞，此时读取的 data_ref_count 的结果状态是可以维持到下面的，
        // 如果为 1，那么表示只有一个存在的 Arc，不需要担心，如果不为1，表示存在多个，不能进行
        let is_unique = arc.data().data_ref_count.load(Ordering::Relaxed) == 1;
        // Release matches Acquire increment in `downgrade`, to make sure any
        // changes to the data_ref_count that come after `downgrade` don't
        // change the is_unique result above.
        arc.data().alloc_ref_count.store(1, Ordering::Release);
        if !is_unique {
            return None;
        }

        fence(Ordering::Acquire);
        unsafe { Some(&mut *arc.data().data.get()) }
    }
}

impl<T> Deref for Arc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.data().data.get() }
    }
}

impl<T> Clone for Arc<T> {
    fn clone(&self) -> Self {
        // TODO: hanlde overflows
        self.data().data_ref_count.fetch_add(1, Ordering::Relaxed);
        Arc { ptr: self.ptr }
    }
}

impl<T> Drop for Arc<T> {
    fn drop(&mut self) {
        if self.data().data_ref_count.fetch_sub(1, Ordering::Relaxed) == 1 {
            fence(Ordering::Acquire);
            unsafe {
                // 这里一定安全嘛？可能有其它的 Weak 此时同时 upgrade? 为了避免这个问题，需要在 Weak upgrade 的时候使用 compare_and_change
                // Safety: The data reference counter is zero,
                // so nothing will access the data anymore.
                ManuallyDrop::drop(self.ptr.as_mut().data.get_mut());

                // 因为所有的 Arc 对应一个 Weak，当没有 Arc 的时候就 Drop 掉对应的 Weak
                drop(Weak { ptr: self.ptr });
            }
        }
    }
}

pub struct Weak<T> {
    ptr: NonNull<ArcData<T>>,
}

unsafe impl<T: Sync + Send> Sync for Weak<T> {}
unsafe impl<T: Sync + Send> Send for Weak<T> {}

impl<T> Weak<T> {
    fn data(&self) -> &ArcData<T> {
        unsafe { self.ptr.as_ref() }
    }

    #[allow(dead_code)]
    pub fn upgrade(&self) -> Option<Arc<T>> {
        let mut n = self.data().data_ref_count.load(Ordering::Relaxed);

        loop {
            if n == 0 {
                return None;
            }

            if let Err(e) = self.data().data_ref_count.compare_exchange_weak(
                n,
                n + 1,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                n = e;
                continue;
            }

            return Some(Arc { ptr: self.ptr });
        }
    }
}

impl<T> Clone for Weak<T> {
    fn clone(&self) -> Self {
        self.data().alloc_ref_count.fetch_add(1, Ordering::Relaxed);
        Weak { ptr: self.ptr }
    }
}

impl<T> Drop for Weak<T> {
    fn drop(&mut self) {
        if self.data().alloc_ref_count.fetch_sub(1, Ordering::Relaxed) == 1 {
            fence(Ordering::Acquire);
            unsafe {
                drop(Box::from_raw(self.ptr.as_ptr()));
            }
        }
    }
}

#[test]
fn test_custom_arc() {
    static NUM_DROPS: AtomicUsize = AtomicUsize::new(0);

    struct DetectDrop;

    impl Drop for DetectDrop {
        fn drop(&mut self) {
            NUM_DROPS.fetch_add(1, Ordering::Release);
        }
    }

    let mut x = Arc::new(("hello", DetectDrop));
    let y = Arc::downgrade(&x);
    let z = Arc::downgrade(&x);

    let t = thread::spawn(move || {
        let y = y.upgrade().unwrap();
        assert_eq!(y.0, "hello");
    });

    assert_eq!(x.0, "hello");

    t.join().unwrap();

    println!("drops num value is: {}", NUM_DROPS.load(Ordering::Relaxed));
    assert_eq!(NUM_DROPS.load(Ordering::Relaxed), 0);
    assert!(z.upgrade().is_some());

    let res = Arc::get_mut(&mut x);
    match res {
        _ => {}
    }

    drop(x);

    assert_eq!(NUM_DROPS.load(Ordering::Relaxed), 1);
}
