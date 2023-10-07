use std::{
    cell::UnsafeCell,
    // collections::VecDeque,
    marker::PhantomData,
    mem::MaybeUninit,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
        // Condvar, Mutex,
    },
    thread::{self, Thread},
};

struct Channel<T> {
    message: UnsafeCell<MaybeUninit<T>>,
    ready: AtomicBool,
}

pub struct Sender<T> {
    channel: Arc<Channel<T>>,
    receiver_thread: Thread,
}

pub struct Receiver<T> {
    channel: Arc<Channel<T>>,
    _no_send: PhantomData<*const ()>,
}

impl<T> Sender<T> {
    /// Safety: Only call this once!
    pub unsafe fn send(self, v: T) {
        (*self.channel.message.get()).write(v);
        self.channel.ready.store(true, Ordering::Release);
        self.receiver_thread.unpark();
    }
}

impl<T> Receiver<T> {
    /// Safety: Only call this once,
    /// and only after is_ready() returns true!
    pub unsafe fn receive(&self) -> T {
        while !self.channel.ready.swap(false, Ordering::Acquire) {
            // panic!("no message available!");
            thread::park();
        }
        (*self.channel.message.get()).assume_init_read()
    }

    // pub fn is_ready(&self) -> bool {
    //     self.channel.ready.load(Ordering::Relaxed)
    // }
}

unsafe impl<T> Sync for Channel<T> where T: Send {}

impl<T> Channel<T> {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            message: UnsafeCell::new(MaybeUninit::uninit()),
            ready: AtomicBool::new(false),
        }
    }
}

impl<T> Drop for Channel<T> {
    fn drop(&mut self) {
        if *self.ready.get_mut() {
            unsafe {
                self.message.get_mut().assume_init_drop();
            }
        }
    }
}

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let a = Arc::new(Channel {
        message: UnsafeCell::new(MaybeUninit::uninit()),
        ready: AtomicBool::new(false),
    });

    (
        Sender {
            channel: a.clone(),
            receiver_thread: thread::current(),
        },
        Receiver {
            channel: a,
            _no_send: PhantomData,
        },
    )
}

#[allow(dead_code)]
pub fn channel_usage() {
    let (sender, receiver) = channel();
    let t = thread::current();
    thread::scope(|s| {
        s.spawn(|| {
            unsafe {
                sender.send("hello cheng");
                // sender.send("hello cheng");
            }
            t.unpark();
        });
    });

    unsafe {
        assert_eq!(receiver.receive(), "hello cheng");
    }
}
