use std::{cell::Cell, marker::PhantomData, rc::Rc, thread};

#[allow(dead_code)]
struct U {
    x: u32,
    // 由于 Cell<()> 不是 sync 类型，那么 PhantomData 也不是，从而限制了 U 也不是 sync 类型
    _not_sync: PhantomData<Cell<()>>,
}

// 也可以主动为某个类型实现对应的 trait，属于我们对编译器的一个承诺
#[allow(dead_code)]
struct RawPointer {
    p: *mut i32,
}

unsafe impl Send for RawPointer {}
unsafe impl Sync for RawPointer {}

#[allow(dead_code)]
pub fn must_be_send() {
    let a = Rc::new([1]);

    // uncommnet to see the errors
    // thread::spawn(move || dbg!(a));
}
