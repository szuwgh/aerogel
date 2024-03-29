#![feature(lazy_cell)]
#![feature(thread_id_value)]
#![feature(thread_local)]

#[macro_use]
pub mod macros;
//mod cell;
mod coroutine;
mod deque;
mod park;
pub mod processor;
mod queue;
mod reactor;
mod task;
use crate::park::Parker;
use crate::processor::Processor;
use crate::processor::{run, EX};
use crate::queue::LocalQueue;
use futures::future::Future;
use futures::task::Context;
use processor::{Executor, Local, Other, Shard};
use std::sync::Arc;
use std::task::Wake;
mod rand;
use crate::queue::LocalScheduler;
use crate::task::new_task;
use crate::task::JoinHandle;
use std::{
    mem,
    pin::Pin,
    task::{Poll, RawWaker, RawWakerVTable, Waker},
};
mod machine;
use crate::machine::ThreadPool;

pub struct Runtime {
    main_p: Arc<Processor>,
}

fn go() {}

fn chan() {}

pub fn spawn<T>(fut: T) -> JoinHandle<T::Output>
where
    T: Future + Send + 'static,
{
    let (task, join) = new_task(0, fut, LocalScheduler);
    EX.with(|ex| {
        //let t = Coroutine::new(Routine::new(RefCell::new(fut.boxed_local())));
        ex.0.push(task);
        ex.0.unpark_one();
    });

    join
}

impl Runtime {
    pub fn new(worker_threads: usize) -> Runtime {
        let mut others: Vec<Other> = Vec::new();
        let mut locals: Vec<Local> = Vec::new();
        for _ in 0..(worker_threads + 1) {
            let park = Parker::new();
            let queue = Arc::new(LocalQueue::new());
            others.push(Other::new(queue.clone(), park.unpark()));
            locals.push(Local::new(queue.clone(), park.clone()));
        }
        let shard = Arc::new(Shard::new(others));
        let mut processors: Vec<Arc<Processor>> = Vec::new();

        for (i, local) in locals.drain(..).enumerate() {
            processors.push(Arc::new(Processor::new(i, shard.clone(), local)));
        }
        ThreadPool::launch(&mut processors);
        Self {
            main_p: processors[0].clone(),
        }
    }

    pub fn block_on<F, T, O>(&mut self, f: F) -> O
    where
        F: Fn() -> T,
        T: Future<Output = O> + 'static,
    {
        let dummpy_waker = get_dummpy_waker();
        let mut cx = Context::from_waker(&dummpy_waker);
        let mut fut = f();
        let mut future = unsafe { Pin::new_unchecked(&mut fut) };
        let cxe = Executor(self.main_p.clone());
        EX.set(&cxe, || loop {
            match Future::poll(future.as_mut(), &mut cx) {
                Poll::Ready(val) => {
                    // cxe.0.schedule();
                    break val;
                }
                Poll::Pending => {
                    cxe.0.schedule();
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
