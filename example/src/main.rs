use aerogel::{spawn, Runtime};
use std::thread;
use std::time::Duration;
fn main() {
    let mut t = Runtime::new(2);
    t.block_on(serve);
}

async fn serve() {
    let f1 = async move {
        println!("f1");
    };
    let f2 = async move {
        println!("f2");
    };
    spawn(f1);
    spawn(f2);
    thread::sleep(Duration::from_secs(5));
}
