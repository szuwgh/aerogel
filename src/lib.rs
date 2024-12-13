#![feature(lazy_cell)]
#![feature(thread_id_value)]
#![feature(thread_local)]

#[macro_use]
pub mod macros;
mod job;
pub mod parallel;
pub mod processor;
mod queue;
mod task;
use crate::job::JobHandle;
use crate::processor::Processor;
use crate::processor::{run, EX};
use crate::queue::LocalQueue;
use crate::task::waker_fn::dummy_waker;
use crossbeam_utils::sync::Parker;
use job::StackJob;
use lazy_static::lazy_static;
use processor::{Executor, Local, Other, Shard};
use std::fmt::Debug;
use std::sync::Arc;
mod rand;
use std::task::Waker;
use std::thread;
mod machine;
use crate::machine::ThreadPool;
use num_cpus;

lazy_static! {
    static ref RUNTIME: Runtime = Runtime::new(num_cpus::get());
}

#[ctor::ctor]
fn init() {
    RUNTIME.init();
}

unsafe impl Sync for Runtime {}
unsafe impl Send for Runtime {}

pub struct Runtime {
    main_p: Arc<Processor>,
    ex: Executor,
    share: Arc<Shard>,
    waker: Waker,
    thead_num: usize,
}

pub fn go<F, R>(f: F) -> JobHandle<F, R>
where
    F: FnOnce() -> R,
    R: Debug,
{
    RUNTIME.go(f)
}

pub(crate) fn get_num() -> usize {
    RUNTIME.thead_num
}

pub fn join<F1, F2, R1, R2>(f1: F1, f2: F2) -> (R1, R2)
where
    F1: FnOnce() -> R1,
    F2: FnOnce() -> R2,
{
    RUNTIME.join(f1, f2)
}

impl Runtime {
    pub fn new(worker_threads: usize) -> Runtime {
        let mut others: Vec<Other> = Vec::new();
        let mut locals: Vec<Local> = Vec::new();
        //这里需要+1 worker_threads+1 线程 一个用于main线程 worker_threads个子线程
        for _ in 0..worker_threads + 1 {
            let park = Parker::new();
            let queue = Arc::new(LocalQueue::new());
            others.push(Other::new(queue.clone(), park.unparker().clone()));
            locals.push(Local::new(queue.clone(), park));
        }
        let shard = Arc::new(Shard::new(others));
        let mut processors: Vec<Arc<Processor>> = Vec::new();

        for (i, local) in locals.drain(..).enumerate() {
            processors.push(Arc::new(Processor::new(i, shard.clone(), local)));
        }
        ThreadPool::launch(&mut processors);
        Self {
            main_p: processors[0].clone(),
            ex: Executor(processors[0].clone()),
            share: shard,
            waker: dummy_waker(),
            thead_num: worker_threads,
        }
    }

    //do nothing
    pub(crate) fn init(&self) {}

    pub fn go<F, R>(&self, f: F) -> JobHandle<F, R>
    where
        F: FnOnce() -> R,
        R: Debug,
    {
        let mut job = JobHandle::new(f); //new_task(0, fut, LocalScheduler);
        self.share.queue.push(job.as_job());
        self.share.unpark_one();
        job
    }

    pub fn join<F1, F2, R1, R2>(&self, f1: F1, f2: F2) -> (R1, R2)
    where
        F1: FnOnce() -> R1,
        F2: FnOnce() -> R2,
    {
        let job1 = StackJob::new(f1); //new_task(0, fut, LocalScheduler);
        let job2 = StackJob::new(f2); //new_task(0, fut, LocalScheduler);
        self.share.queue.push(job1.as_job());
        self.share.queue.push(job2.as_job());
        self.share.unpark_two();
        while job2.lock.wait() || job1.lock.wait() {
            if let Some(job) = self.main_p.steal_job() {
                job.run();
            } else {
                thread::yield_now();
            }
        }
        (job1.get_res(), job2.get_res())
    }

    // pub fn block_on<F, T, O>(self, f: F) -> O
    // where
    //     F: Fn() -> T,
    //     T: Future<Output = O> + 'static,
    // {
    //     let dummpy_waker = dummy_waker();
    //     let mut cx = Context::from_waker(&dummpy_waker);
    //     let mut fut = f();
    //     let mut future = unsafe { Pin::new_unchecked(&mut fut) };
    //     EX.set(&self.ex, || loop {
    //         match Future::poll(future.as_mut(), &mut cx) {
    //             Poll::Ready(val) => {
    //                 break val;
    //             }
    //             Poll::Pending => {
    //                 self.ex.0.schedule();
    //             }
    //         };
    //     })
    // }
}

// fn get_dummpy_waker() -> Waker {
//     Waker::from(Arc::new(Helper(|| {})))
// }

// struct Helper<F>(F);

// impl<F: Fn() + Send + Sync + 'static> Wake for Helper<F> {
//     fn wake(self: Arc<Self>) {
//         (self.0)();
//     }

//     fn wake_by_ref(self: &Arc<Self>) {
//         (self.0)();
//     }
// }

mod tests {
    use super::*;

    #[test]
    fn test_processor() {
        let mut a = vec![1, 2, 3, 4, 5];
        drain(&mut a);
        println!("{:?}", a);
    }

    fn drain(a: &mut Vec<i32>) {
        for _ in a.drain(..a.len() - 1) {}
    }
}
