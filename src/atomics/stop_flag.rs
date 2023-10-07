use std::{
    io::stdin,
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::Duration,
};

static STOP: AtomicBool = AtomicBool::new(false);

#[allow(dead_code)]
pub fn stop_flag() {
    let background_t1 = thread::spawn(|| {
        while !STOP.load(Ordering::Relaxed) {
            println!("do something in the background.");
            thread::sleep(Duration::from_secs(3));
        }
    });

    for line in stdin().lines() {
        match line.unwrap().as_str() {
            "help" => println!("commands: help, stop"),
            "stop" => break,
            cmd => println!("unknown command {cmd:?}"),
        }
    }

    STOP.store(true, Ordering::Relaxed);

    background_t1.join().unwrap();
}
