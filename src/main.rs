#![feature(arc_into_inner)]
use std::{
    hint::black_box,
    sync::atomic::{AtomicU16, AtomicU64, AtomicUsize, Ordering},
    thread,
    time::Instant,
};

mod arc;
pub mod atomics;
mod carton;
mod channel;
mod cond_var;
mod condition_var;
mod interior_mutability;
mod mutex;
mod mutex_usage;
pub mod ordering;
mod parking;
mod reference_counting;
mod rwlock;
mod scoped_thread;
mod send_sync_trait;
mod shared_data;
mod spin_lock;

fn main() {
    // scoped_thread::scoped_thread();
    // shared_data::shared_memory();
    // reference_counting::reference_counting();
    // mutex::mutex_usage();
    // parking::thread_parking();
    // condition_var::condition_var();
    // atomics::stop_flag::stop_flag();
    // atomics::progress_report::display_update_progress();
    // ordering::relaxed::relaxed_ordering();
    // ordering::release_acquire::release_acquire_ordering();
    // ordering::mutex::custom_lock();
    // ordering::happen_before::happen_beofore();
    // ordering::fences::fences();
    // spin_lock::spin_lock_usage();
    // channel::channel_usage();
}

#[test]
fn test_atomic() {
    static A: AtomicUsize = AtomicUsize::new(0);
    thread::spawn(|| loop {
        black_box(A.store(0, Ordering::Relaxed));
    });

    let start = Instant::now();
    for _ in 0..1_000_000_000 {
        black_box(A.load(Ordering::Relaxed));
    }

    println!("{:?}", start.elapsed());
}

#[test]
fn test_group_atomics() {
    static A: [AtomicU64; 3] = [AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0)];
    black_box(&A);
    thread::spawn(|| loop {
        A[0].store(0, Ordering::Relaxed);
        A[2].store(0, Ordering::Relaxed);
    });
    let start = Instant::now();
    for _ in 0..1_000_000_000 {
        black_box(A[1].load(Ordering::Relaxed));
    }
    println!("{:?}", start.elapsed());
}

#[test]
fn test_split_atomics() {
    #[repr(align(64))] // This struct must be 64-byte aligned.
    struct Aligned(AtomicU64);

    static A: [Aligned; 3] = [
        Aligned(AtomicU64::new(0)),
        Aligned(AtomicU64::new(0)),
        Aligned(AtomicU64::new(0)),
    ];

    black_box(&A);
    thread::spawn(|| loop {
        A[0].0.store(1, Ordering::Relaxed);
        A[2].0.store(1, Ordering::Relaxed);
    });
    let start = Instant::now();
    for _ in 0..1_000_000_000 {
        black_box(A[1].0.load(Ordering::Relaxed));
    }
    println!("{:?}", start.elapsed());
}

#[allow(dead_code)]
fn calculate_average() {
    let nums = Vec::from_iter(0..0);

    let h = thread::spawn(move || {
        let len = nums.len();
        let sum = nums.iter().sum::<usize>();

        sum / len
    });
    let average = h.join().unwrap();
    println!("average: {average}");
}

// use thread::Builder api to create thread
#[allow(dead_code)]
fn create_thread() -> thread::JoinHandle<()> {
    let t_builder = thread::Builder::new();
    let thread = t_builder
        .name("a calculate thread".to_string())
        .spawn(move || {})
        .unwrap();

    thread
}
