#![feature(lazy_cell)]
#![feature(thread_id_value)]
#![feature(thread_local)]

#[macro_use]
pub mod macros;
pub mod processor;
mod queue;
mod task;
use crate::processor::Processor;
use crate::processor::{run, EX};
use crate::queue::LocalQueue;
pub use async_channel::bounded as channel;
pub use async_channel::Receiver;
pub use async_channel::Sender;
pub use async_lock::RwLock as AsyncRwLock;
pub use async_lock::Semaphore;
use crossbeam_utils::sync::Parker;
use futures::future::Future;
use futures::task::Context;
use processor::{Executor, Local, Other, Shard};
use std::sync::Arc;
use std::task::Wake;
mod rand;
use crate::queue::LocalScheduler;
use crate::task::new_task;
use crate::task::JoinHandle;
use once_cell::sync::Lazy;
use std::{
    pin::Pin,
    task::{Poll, Waker},
};
mod machine;
use crate::machine::ThreadPool;

pub(crate) static RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    // Runtime::new(3)
    Runtime::new(1)
});

unsafe impl Sync for Runtime {}
unsafe impl Send for Runtime {}

pub struct Runtime {
    main_p: Arc<Processor>,
    ex: Executor,
    share: Arc<Shard>,
}

pub fn go<T>(fut: T) -> JoinHandle<T::Output>
where
    T: Future + Send + 'static,
{
    spawn(fut)
}

pub fn chan<T>(cap: usize) -> (Sender<T>, Receiver<T>) {
    channel(cap)
}

pub fn spawn<T>(fut: T) -> JoinHandle<T::Output>
where
    T: Future + Send + 'static,
{
    RUNTIME.spawn(fut)
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
        }
    }

    pub fn spawn<T>(&self, fut: T) -> JoinHandle<T::Output>
    where
        T: Future + Send + 'static,
    {
        let (task, join) = new_task(0, fut, LocalScheduler);
        self.share.queue.push(task);
        self.share.unpark_one();
        join
    }

    pub fn block_on<F, T, O>(self, f: F) -> O
    where
        F: Fn() -> T,
        T: Future<Output = O> + 'static,
    {
        let dummpy_waker = get_dummpy_waker();
        let mut cx = Context::from_waker(&dummpy_waker);
        let mut fut = f();
        let mut future = unsafe { Pin::new_unchecked(&mut fut) };
        EX.set(&self.ex, || loop {
            match Future::poll(future.as_mut(), &mut cx) {
                Poll::Ready(val) => {
                    break val;
                }
                Poll::Pending => {
                    self.ex.0.schedule();
                }
            };
        })
    }
}

fn get_dummpy_waker() -> Waker {
    Waker::from(Arc::new(Helper(|| {})))
}

struct Helper<F>(F);

impl<F: Fn() + Send + Sync + 'static> Wake for Helper<F> {
    fn wake(self: Arc<Self>) {
        (self.0)();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        (self.0)();
    }
}

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
