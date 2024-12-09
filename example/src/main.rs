use aerogel::{spawn, Runtime};
use std::sync::atomic::AtomicI32;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
fn main() {
    let mut t = Runtime::new(5);

    t.block_on(serve);

    thread::sleep(Duration::from_secs(10));
}

async fn serve() {
    //loop {
    let check = Arc::new(AtomicI32::new(0));
    for i in 0..100000 {
        let c = check.clone();
        let f = async move {
            println!("{:?},{}", thread::current().id(), i);
            c.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            // for _ in 0..1000 {
            //     let a = 1 + 1;
            // }
            // thread::sleep(Duration::from_micros(50));
        };
        spawn(f);
    }
    thread::sleep(Duration::from_secs(5));
    println!("check:{}", check.load(std::sync::atomic::Ordering::SeqCst));
    // }
}
