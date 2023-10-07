use std::thread;

// 对于一般的 thread::spawn 来说，不能捕获闭包外面的引用，这是因为 spawn 的线程的生命周期为 static
// 我们不确定它何时能结束。
// 如果我们希望它可以捕获外接的变量，我们必须知道它在何时结束，此时可以利用 thread::scope API 去实现这个需求，
// 但缺点是，thread::scope 会利用 park API 阻塞当前主线程的执行，只有当 scope 内所有的线程结束之后它才会恢复主线程的执行。
#[allow(dead_code)]
pub fn scoped_thread() {
    #[allow(unused_mut)]
    let mut nums = vec![1, 2, 3, 4, 5];

    thread::scope(|s| {
        s.spawn(|| {
            println!("length of nums is {}", nums.len());
        });

        s.spawn(|| {
            for n in &nums {
                println!("{n}");
            }
        });

        // 不可以同时借用两个可变引用
        // s.spawn(|| {
        //     nums.push(6);
        // });

        // s.spawn(|| {
        //     nums.push(7);
        // });
    });

    println!("resume main thread execution.");
}
