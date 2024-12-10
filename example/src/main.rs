use aerogel::bounded;
use aerogel::go;
use aerogel::{Runtime, spawn};
use std::sync::Arc;
use std::sync::atomic::AtomicI32;
use std::thread;
use std::time::Duration;
use tokio::runtime::Builder;
use tokio::sync::mpsc;
use tokio::time::{self};
fn main() {
    // let mut t = Runtime::new(2);

    //t.block_on(serve);
    let (s, r) = async_channel::bounded::<usize>(1); //bounded::<usize>(0);
    // let s1 = Arc::new(s);
    // let s3 = s1.clone();

    // let (s, mut r) = mpsc::channel::<usize>(1);

    // let rt = Builder::new_multi_thread()
    //     .worker_threads(1)
    //     .build()
    //     .expect("Failed to create runtime");
    go(async move {
        //loop {
        let check = Arc::new(AtomicI32::new(0));
        for i in 0..2 {
            let c = check.clone();
            let _ = s.send(i).await;

            println!("send:{}", i);
            // async {
            //     thread::sleep(Duration::from_secs(1));
            // }
            // .await;
        }
        // thread::sleep(Duration::from_secs(5));
        println!("check:{}", check.load(std::sync::atomic::Ordering::SeqCst));
        // }
    });

    go(async move {
        thread::sleep(Duration::from_secs(1));
        while let Ok(s) = r.recv().await {
            println!("re:{}", s);
        }
    });

    // go(async move {
    //     //loop {
    //     let check = Arc::new(AtomicI32::new(0));
    //     for i in 0..10 {
    //         let c = check.clone();
    //         async {
    //             s3.send(i).unwrap();
    //         }
    //         .await;

    //         println!("send:{}", i);
    //         // async {
    //         //     thread::sleep(Duration::from_secs(1));
    //         // }
    //         // .await;
    //     }
    //     thread::sleep(Duration::from_secs(5));
    //     println!("check:{}", check.load(std::sync::atomic::Ordering::SeqCst));
    //     // }
    // });
    // go(async move {
    //     while let Ok(s) = async { r.recv() }.await {
    //         println!("re:{}", s);
    //     }
    // });
    // //thread::sleep(Duration::from_secs(2));

    // thread::sleep(Duration::from_secs(10));

    thread::sleep(Duration::from_secs(10));
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
