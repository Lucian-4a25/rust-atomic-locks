use std::{
    sync::{
        atomic::{AtomicU32, Ordering},
        Once,
    },
    thread,
    time::Duration,
};

static X: AtomicU32 = AtomicU32::new(0);
static ONCE: Once = Once::new();
static mut TEMP: u32 = 0;
// 用标志位判断是否已经初始化的方式不可行，因为如果将标志位的值进行了改变，但未改变真正的 X，
// 此时如果直接返回 X 依然会导致不正确的结果；依然会导致同时调用计算懒值的过程。
// static IF_INITED: AtomicBool = AtomicBool::new(false);

// 这里的问题是，如果希望懒加载一次，多线程同时进入这个方法就可能会导致计算多次并互相覆盖写入的结果
// 我们可以改用 compare_and_exchange 这样的原子操作 api，使得只会写入一份结果

// 对于多次执行的问题，可以借用 Once API
#[allow(dead_code)]
pub fn get_x() -> u32 {
    // let inited = IF_INITED.load(Ordering::Relaxed);

    let v = X.load(Ordering::Relaxed);
    if v == 0 {
        let new_v = unsafe {
            ONCE.call_once(|| {
                TEMP = calc_new_val();
            });
            TEMP
        };

        match X.compare_exchange_weak(0, new_v, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => {
                return new_v;
            }
            Err(v) => return v,
        }
    } else {
        v
    }
}

pub fn calc_new_val() -> u32 {
    thread::sleep(Duration::from_secs(3));
    33
}
