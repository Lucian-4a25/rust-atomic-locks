use std::{
    sync::atomic::{AtomicUsize, Ordering},
    thread,
};

static X: AtomicUsize = AtomicUsize::new(0);

#[allow(dead_code)]
pub fn relaxed_ordering() {
    let t1 = thread::spawn(a);
    let t2 = thread::spawn(a2);
    let t3 = thread::spawn(b);

    t1.join().unwrap();
    t2.join().unwrap();
    t3.join().unwrap();
}

pub fn b() {
    let a = X.load(Ordering::Relaxed);
    let b = X.load(Ordering::Relaxed);
    let c = X.load(Ordering::Relaxed);
    let d = X.load(Ordering::Relaxed);
    let e = X.load(Ordering::Relaxed);
    let f = X.load(Ordering::Relaxed);
    let g = X.load(Ordering::Relaxed);

    println!("a: {a}, b: {b}, c: {c}, d: {d}, e: {e}, f: {f}, g: {g}");
}

pub fn a() {
    X.fetch_add(5, Ordering::Relaxed);
}

pub fn a2() {
    X.fetch_add(10, Ordering::Relaxed);
}
