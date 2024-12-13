use aerogel::go;

use aerogel::join;
use aerogel::parallel::IntoParallelIterator;
use aerogel::parallel::ParallelIterator as a;
use futures::StreamExt;
use futures::future::FutureExt;
use futures::future::poll_fn;
use futures::join;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use std::pin::pin;
use std::sync::Arc;
use std::sync::atomic::AtomicI32;
use std::thread;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
fn main() {
    let a = vec![1.0; 1000000];

    let time1 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    //  let b = &a[..];
    let c = (&a[..]).par_iter().sum::<f64>();
    let time2 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    println!("{}", c);
    println!("par:{}", time2 - time1);

    let time1 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    //  let b = &a[..];
    let c = (&a[..]).par_iter().sum::<f64>();
    let time2 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    println!("{}", c);
    println!("par:{}", time2 - time1);

    let time1 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    let c = (&a[..]).into_aer_iter().sum::<f64>();
    let time2 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    println!("{}", c);
    println!("aer:{}", time2 - time1);

    let time1 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    //  let b = &a[..];
    let c = (&a[..]).iter().sum::<f64>();
    let time2 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    println!("{}", c);
    println!("{}", time2 - time1);

    // let time1 = SystemTime::now()
    //     .duration_since(UNIX_EPOCH)
    //     .unwrap()
    //     .as_nanos();

    // let c = (&a[..]).par_iter().sum::<f64>();
    // let time2 = SystemTime::now()
    //     .duration_since(UNIX_EPOCH)
    //     .unwrap()
    //     .as_nanos();
    // println!("{}", c);
    // println!("par:{}", time2 - time1);

    // let time1 = SystemTime::now()
    //     .duration_since(UNIX_EPOCH)
    //     .unwrap()
    //     .as_nanos();

    // let c = (&a[..]).par_iter().sum::<f64>();
    // let time2 = SystemTime::now()
    //     .duration_since(UNIX_EPOCH)
    //     .unwrap()
    //     .as_nanos();
    // println!("{}", c);
    // println!("par:{}", time2 - time1);

    // let time1 = SystemTime::now()
    //     .duration_since(UNIX_EPOCH)
    //     .unwrap()
    //     .as_nanos();

    // let c = (&a[..]).into_aer_iter().sum::<f64>();
    // let time2 = SystemTime::now()
    //     .duration_since(UNIX_EPOCH)
    //     .unwrap()
    //     .as_nanos();
    // println!("{}", c);
    // println!("aer:{}", time2 - time1);

    // let c = (&a[..]).par_iter().sum::<f64>();
    // let time2 = SystemTime::now()
    //     .duration_since(UNIX_EPOCH)
    //     .unwrap()
    //     .as_nanos();
    // println!("{}", c);
    // println!("par:{}", time2 - time1);

    // let time1 = SystemTime::now()
    //     .duration_since(UNIX_EPOCH)
    //     .unwrap()
    //     .as_nanos();

    // let c = (&a[..]).into_aer_iter().sum::<f64>();
    // let time2 = SystemTime::now()
    //     .duration_since(UNIX_EPOCH)
    //     .unwrap()
    //     .as_nanos();
    // println!("{}", c);
    // println!("aer:{}", time2 - time1);

    // let time1 = SystemTime::now()
    //     .duration_since(UNIX_EPOCH)
    //     .unwrap()
    //     .as_nanos();

    // //  let b = &a[..];
    // let c = (&a[..]).into_aer_iter().sum::<usize>();
    // let time2 = SystemTime::now()
    //     .duration_since(UNIX_EPOCH)
    //     .unwrap()
    //     .as_nanos();
    // println!("{}", c);
    // println!("{}", time2 - time1);

    // let time1 = SystemTime::now()
    //     .duration_since(UNIX_EPOCH)
    //     .unwrap()
    //     .as_nanos();
    // //  let b = &a[..];
    // let c = (&a[..]).into_aer_iter().sum::<usize>();
    // let time2 = SystemTime::now()
    //     .duration_since(UNIX_EPOCH)
    //     .unwrap()
    //     .as_nanos();
    // println!("{}", c);
    // println!("{}", time2 - time1);
    // println!("===================");

    // let time1 = SystemTime::now()
    //     .duration_since(UNIX_EPOCH)
    //     .unwrap()
    //     .as_nanos();

    // //  let b = &a[..];
    // let c = (&a[..]).iter().sum::<f64>();
    // let time2 = SystemTime::now()
    //     .duration_since(UNIX_EPOCH)
    //     .unwrap()
    //     .as_nanos();
    // println!("{}", c);
    // println!("{}", time2 - time1);

    // let time1 = SystemTime::now()
    //     .duration_since(UNIX_EPOCH)
    //     .unwrap()
    //     .as_nanos();

    // //  let b = &a[..];
    // let c = (&a[..]).par_iter().sum::<usize>();
    // let time2 = SystemTime::now()
    //     .duration_since(UNIX_EPOCH)
    //     .unwrap()
    //     .as_nanos();
    // println!("{}", c);
    // println!("{}", time2 - time1);

    // let time1 = SystemTime::now()
    //     .duration_since(UNIX_EPOCH)
    //     .unwrap()
    //     .as_nanos();
    // //  let b = &a[..];
    // let c = (&a[..]).par_iter().sum::<usize>();
    // let time2 = SystemTime::now()
    //     .duration_since(UNIX_EPOCH)
    //     .unwrap()
    //     .as_nanos();
    // println!("{}", c);
    // println!("{}", time2 - time1);
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
        // spawn(f);
    }
    thread::sleep(Duration::from_secs(5));
    println!("check:{}", check.load(std::sync::atomic::Ordering::SeqCst));
    // }
}
