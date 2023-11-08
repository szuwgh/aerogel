use aerogel::{spawn, Runtime};
use std::thread;
use std::time::Duration;

fn main() {
    let mut t = Runtime::new(5);
    t.block_on(serve);
    // thread::sleep(Duration::from_secs(2));
}

async fn serve() {
    loop {
        for i in 0..1000 {
            let f = async move {
                print!("{}", i);
                // for _ in 0..1000 {
                //     let a = 1 + 1;
                // }
                thread::sleep(Duration::from_micros(50));
            };
            spawn(f);
        }
    }
}
