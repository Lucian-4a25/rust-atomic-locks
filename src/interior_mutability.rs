use std::cell::{Cell, RefCell};

// 使用 Cell 只能整个替换取出它内部的值 (take API，要求实现 Default，replace API，传递被替换的值)，或者将对应的值复制取出来 (前提是该类型实现了 Copy)
// 除此之外，它只能用于单线程的情况
#[allow(dead_code)]
pub fn cell_mutability(a: &Cell<i32>, b: &Cell<i32>) {
    // get API 的调用前提是 T 实现了 Copy
    let before = a.get();
    b.set(b.get() + 1);
    let after = a.get();
    if before != after {
        println!("do something");
    }
}

// 只能先取出来，再整体替换
#[allow(dead_code)]
pub fn cell_mutability_vec(a: &Cell<Vec<u32>>) {
    let mut v = a.take();
    v.push(3);

    a.set(v);
}

// 只能用于单线程的情况
#[allow(dead_code)]
pub fn ref_cell_mutability(a: &RefCell<Vec<u32>>) {
    a.borrow_mut().push(2);
    a.borrow_mut().push(3);
}

// RwLock 是一个多线程版本的 RefCell

// Atomic types 表示的是多线程版本的 Cell

// 上述所有类型内部都是通过 UnsafeCell 调用原始指针支持结合 unsafe 实现的
