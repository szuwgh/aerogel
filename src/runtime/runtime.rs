use crate::runtime::job::JobAdapter;
use crate::runtime::job::JobHandle;
use crate::runtime::job::StackJob;
use crate::runtime::processor::Processor;
use crate::runtime::processor::{run, EX};
use crate::runtime::processor::{Executor, Local, Other, Shard};
use crate::runtime::queue::LocalQueue;
use crate::runtime::thpool::ThreadPool;
use crossbeam_utils::sync::Parker;
use lazy_static::lazy_static;
use num_cpus;
use std::error::Error;
use std::fmt;
use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::Waker;
use std::thread;

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
            //  所以这段代码体现的是：
            //  当前线程既是“等待者”
            //  也是“工作者
            if let Some(job) = self.main_p.steal_job() {
                job.run();
            } else {
                thread::yield_now();
            }
        }
        (job1.get_res(), job2.get_res())
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub enum RuntimeType {
    #[default]
    Default,
    Tokio,
}

#[derive(Debug, Clone)]
pub struct RuntimeError {
    message: String,
}

impl RuntimeError {
    pub fn join(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "runtime task failed: {}", self.message)
    }
}

impl Error for RuntimeError {}

pub trait SpawnRuntime: Send + Sync + 'static {
    type Join<T>: Future<Output = Result<T, RuntimeError>> + Send
    where
        T: Send + 'static;

    fn spawn<Fut>(&self, fut: Fut) -> Self::Join<Fut::Output>
    where
        Fut: Future + Send + 'static,
        Fut::Output: Send + 'static;
}

#[derive(Clone, Default)]
pub struct TokioRuntime;

#[derive(Clone, Default)]
pub struct InlineRuntime;

#[derive(Clone, Default)]
pub struct GlobalRuntime;
