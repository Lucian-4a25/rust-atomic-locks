use std::{
    sync::atomic::{AtomicBool, Ordering},
    thread,
};

static LOCKED: AtomicBool = AtomicBool::new(false);
static mut DATA: String = String::new();

#[allow(dead_code)]
pub fn custom_lock() {
    thread::scope(|s| {
        for _ in 0..100 {
            s.spawn(|| {
                // compare_exchange 有两个 Ordering, 一个是用于 compare 的条件成功的情况，一个是失败的情况。
                // 如果 copmare 成功，需要涉及到读取和修改两个行为的Ordering，success 的 ordering 的值如果使用 Acquire，
                // 表示读取的时候用 Acquire，写的时候使用 Relaxed. 同样地，如果使用 Release，表示写的时候使用 Release，读取使用 Relaxed.
                if LOCKED
                    .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
                    .is_ok()
                {
                    // 由于 Release、Acquire 的保证，这里面的行为一定是在另外的线程 Release 后，也就是不会被处理器随意移动
                    // 以此我们知道，DATA只可能某个时间段只有一个访问对象
                    unsafe {
                        DATA.push('!');
                    }
                    LOCKED.store(false, Ordering::Release);
                }
            });
        }
    });

    unsafe {
        println!("the resutl of DATA: {}, len: {}", DATA.as_str(), DATA.len());
    }
}
