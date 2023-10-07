use std::{collections::VecDeque, sync::Mutex, thread, time::Duration};

// 虽然这种简单的生产消费者的模式看上去可以解决问题，但是这样做并不是十分有效率。
// 因为即使是利好的情况，即消费者先 parking，再被生产者 unparking，消费者在消费完之后
// 依然进行一次加锁解锁检查，然后再进入 parking 等待被唤醒。
// 如果是刚生产马上被消费，即使是这种完美的情况，消费者会第二次进行加锁解锁，进行 parking，并被之前生产者产生的 unparking
// 唤醒，再进行一次加锁解锁，这样导致效率不是很高。
// 而对于更加复杂的场景，如多消费者，单生产者，目前的方式容易导致多次没必要的唤醒消费者。

// 因此，对于生产消费者模型，使用 parking 的方式不够有效率，因为生产者和消费者之间没有互相通知的方式。
// 此时我们需要用到 condition variable 去解决这个问题。
#[allow(dead_code)]
pub fn thread_parking() {
    let queue = Mutex::new(VecDeque::new());

    thread::scope(|s| loop {
        println!("running scope closure...");
        // consuming thread
        let consumer_thread = s.spawn(|| {
            println!("running consumer closure...");
            let item = queue.lock().unwrap().pop_front();
            if let Some(item) = item {
                dbg!(item);
            } else {
                thread::park();
            }
        });

        // 生产提供给消费线程
        for i in 0..20 {
            println!("running producer closure...");
            queue.lock().unwrap().push_back(i);
            consumer_thread.thread().unpark();
            thread::sleep(Duration::from_secs(1));
        }
        // consumer_thread.thread().unpark();
    });
}
