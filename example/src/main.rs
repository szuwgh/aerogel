use aerogel::{spawn, Runtime};
use std::sync::atomic::AtomicI32;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use aerogel::go;
use aerogel::bounded;

fn main() {
   // let mut t = Runtime::new(2);

    //t.block_on(serve);
    let (s,r) = bounded::<usize>(0);
    go(async move {
        //loop {
        let check = Arc::new(AtomicI32::new(0));
        for i in 0..100 {
            let c = check.clone();
            s.send(i).unwrap();
            println!("send:{}",i );
            //thread::sleep(Duration::from_secs(1));
        }
        thread::sleep(Duration::from_secs(5));
        println!("check:{}", check.load(std::sync::atomic::Ordering::SeqCst));
        // }
    });
    thread::sleep(Duration::from_secs(2));
    while let Ok(s) = r.recv() {
        println!("re:{}",s);

    }

    // thread::sleep(Duration::from_secs(10));

  //  thread::sleep(Duration::from_secs(10));
}

async fn serve() {
    //loop {
    let check = Arc::new(AtomicI32::new(0));
    for i in 0..110 {
        let c = check.clone();
        let f = async move {
            println!("{:?},{}", thread::current().id(), i);
            c.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            // for _ in 0..1000 {
            //     let a = 1 + 1;
            // }
            thread::sleep(Duration::from_micros(100));
        };
        spawn(f);
    }
    thread::sleep(Duration::from_secs(5));
    println!("check:{}", check.load(std::sync::atomic::Ordering::SeqCst));
    // }
}
