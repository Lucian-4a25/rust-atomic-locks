use std::thread;
static X: [u32; 3] = [1, 2, 3];

#[allow(dead_code)]
pub fn shared_memory() {
    // 引用类型实现了 Copy，所以可以随意的 move 它
    let x: &'static [u32; 3] = Box::leak(Box::new([1, 2, 3]));

    let h1 = thread::spawn(move || {
        dbg!(&X);
    });

    let h2 = thread::spawn(move || {
        dbg!(&X);
    });

    let h3 = thread::spawn(move || {
        for i in x {
            println!("{i}");
        }
    });

    let h4 = thread::spawn(move || {
        for i in x {
            println!("{i}");
        }
    });

    h1.join().unwrap();
    h2.join().unwrap();
    h3.join().unwrap();
    h4.join().unwrap();
}
