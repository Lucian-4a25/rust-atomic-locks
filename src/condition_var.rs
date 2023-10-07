use std::{
    collections::VecDeque,
    sync::{Arc, Condvar, Mutex},
    thread,
};

// 使用 condvar 去解决生产消费者模型中的问题，通过 mutex 结合 condvar 的方式。
// 对于 condvar 来说，释放锁和等待被唤醒是一个原子操作，所以不会在释放锁之后错过被唤醒的机会。
// 这样，condvar 被唤醒时，必然有数据可以消耗。
// 通过将释放锁和睡眠串联为原子操作，保证了在释放锁和休眠之间不会有其它无效的唤醒操作，避免多余的锁的开销。
// 但是，这样也会导致在获取锁直至睡眠前，都不会释放锁，此时生产者无法在这个时间内获取锁生产数据。
// 因为数据是有锁加持的，只能存在一个修改者，所以这种情况也不会影响效率。
#[allow(dead_code)]
pub fn condition_var() {
    let not_empty = Arc::new(Condvar::new());
    let queue = Arc::new(Mutex::new(VecDeque::new()));
    let consumer_queue = queue.clone();
    let consumer_not_empty = not_empty.clone();

    thread::scope(|s| {
        s.spawn(move || {
            let mut lock_guide = consumer_queue.lock().unwrap();

            loop {
                if let Some(d) = lock_guide.pop_front() {
                    dbg!(d);
                } else {
                    // wait api 消耗锁，这个 api 会同时释放锁以及进行休眠
                    lock_guide = consumer_not_empty.wait(lock_guide).unwrap();
                }
            }
        });

        for i in 0..20 {
            let queue = queue.clone();
            let not_empty = not_empty.clone();
            s.spawn(move || {
                queue.lock().unwrap().push_back(i);
                not_empty.notify_one();
            });
        }
    });
}
