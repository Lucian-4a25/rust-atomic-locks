use std::{
    cmp::max,
    mem::{align_of, size_of},
    ops::{Deref, DerefMut},
    ptr,
};

pub struct Carton<T>(ptr::NonNull<T>);

// see: https://doc.rust-lang.org/nomicon/send-and-sync.html
// 一句话解释什么是 Send 和 Sync
// Send: 一个数据类型可以安全的被发送给另外一个线程
// Sync: 一个数据类型可以安全的在线程间被共享 (T 是 Sync 的前提是 &T 是 Send) (这里的共享指的是，复制 &T 在多个线程中不会产生同时写入的问题)

// (raw pointers)原始指针不是 Send、Sync，因为我们需要主动的保证它们的安全性
// UnsafeCell 不是 Sync (因为使用 UnsafeCell 允许我们同时拥有多个可写，如果在多个线程之间共享 &UnsafeCell，会导致同时存在多个&mut T)
// Rc 不是 Sync 和 Send (因为它的引用计数是共享的，并未加上写入锁，所以不能安全的在多个线程共享的 Send)
// MutexGuard 不是 Send，MutexGuard 的实现使用的库要求使用者不能尝试在一个线程上获得锁，然后将这个锁在另外一个线程中释放，所以它不是 Send，但是 MutexGuide 可以是 Sync，因为 &MutexGuide 的 drop 不会影响锁的释放。

impl<T> Carton<T> {
    #[allow(dead_code)]
    pub fn new(value: T) -> Self {
        // 在堆上分配足够的内存给 T
        assert_ne!(
            0,
            size_of::<T>(),
            "zero-sized tpyes are out of scope of this example"
        );

        let mut memptr: *mut T = ptr::null_mut();
        unsafe {
            let ret = libc::posix_memalign(
                (&mut memptr as *mut *mut T).cast(),
                max(align_of::<T>(), size_of::<usize>()),
                size_of::<T>(),
            );
            assert_eq!(ret, 0, "Failed to allocate or invalid alignment!");
        }

        let ptr =
            { ptr::NonNull::new(memptr).expect("Guaranteed non-null if posix_memalign returns 0") };

        // 将值从栈移动到堆中指向的区域
        unsafe {
            ptr.as_ptr().write(value);
        }

        Self(ptr)
    }
}

impl<T> Deref for Carton<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref() }
    }
}

impl<T> DerefMut for Carton<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.0.as_mut() }
    }
}

// Safety: No one besides us has the raw pointer, so we can safely transfer the
// Carton to another thread if T can be safely transferred.
// 除了一个变量没有互斥锁的保证下和其它变量共享一个可修改的状态值的情况外，它都可以安全的转为 Send
// 由于 Carton 独享 raw_pointer 的值，所以不需要担心和其它变量共享这个指针，此时可以安全的转换为 Send
unsafe impl<T> Send for Carton<T> where T: Send {}

// Safety: Since there exists a public way to go from a `&Carton<T>` to a `&T`
// in an unsynchronized fashion (such as `Deref`), then `Carton<T>` can't be
// `Sync` if `T` isn't.
// Conversely, `Carton` itself does not use any interior mutability whatsoever:
// all the mutations are performed through an exclusive reference (`&mut`). This
// means it suffices that `T` be `Sync` for `Carton<T>` to be `Sync`:
// Sync 特性是用来保证在多线程环境下，一个类型的值可以被安全地共享，这里的共享指的是多个线程同时读取或写入同一个值。
// 为了保证安全，Rust 的借用检查器会强制要求在写入指针时必须使用 &mut Carton，而可变引用必须是独占的，因此不会出现安全问题。
// 在我们的例子中，有一个非同步的API方法可以从 &Carton<T> 转换为 &T, 所以 &Carton<T> 要变为 Sync 的前提是 &T 是 Sync。
// 其次，Carton 并未使用任何 interior mutablity(内部可变性，使用它可以允许同时存在多个可写引用)，也就是说针对它的所有修改都必须通过
// &mut，这意味着只要 T 是 Sync，那么 Carton<T> 也可以是 Sync.
unsafe impl<T> Sync for Carton<T> where T: Sync {}

impl<T> Drop for Carton<T> {
    fn drop(&mut self) {
        unsafe { libc::free(self.0.as_ptr().cast()) }
    }
}
