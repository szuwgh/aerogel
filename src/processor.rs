use crate::queue::LocalQueue;
use crate::queue::LQ_HALF_SIZE;
use crate::queue::LQ_SIZE;
use crate::queue::{self, GlobalQueue};
use crate::rand::RandomOrder;
use crate::rand::{seed, FastRand};
use core::cmp;
use crossbeam_deque::Steal;
use crossbeam_utils::sync::Parker;
use crossbeam_utils::sync::Unparker;
use futures::future::Future;
use futures::FutureExt;
pub(crate) use parking_lot::Mutex;
use scoped_tls::scoped_thread_local;
use std::cell::{RefCell, UnsafeCell};
use std::rc::Rc;
use std::sync::Arc;

use crate::queue::Coroutine;
use crate::queue::LocalScheduler;
use crate::task::new_task;
use crate::task::JoinHandle;
use core::cell::Cell;

scoped_thread_local!(pub(crate) static EX: Executor);
pub struct Executor(pub(crate) Arc<Processor>);

pub(crate) fn run(p: Arc<Processor>) {
    let cx = Executor(p);
    EX.set(&cx, || loop {
        //循环调度
        cx.0.schedule();
    });
}

unsafe impl Send for Processor {}
unsafe impl Sync for Processor {}

pub(crate) struct Idle {
    // sleeping machine
    sleepers: Mutex<Vec<usize>>,
}

impl Idle {
    fn new() -> Idle {
        Self {
            sleepers: Mutex::new(Vec::new()),
        }
    }

    pub(crate) fn processor_to_parked(&self, p: usize) {
        let mut sleepers = self.sleepers.lock();
        sleepers.push(p);
    }

    pub(crate) fn processor_to_notify(&self) -> Option<usize> {
        let mut sleepers = self.sleepers.lock();
        sleepers.pop()
    }
}

//共享数据
pub(crate) struct Shard {
    others: Box<[Other]>,

    //全局队列
    pub(crate) queue: Arc<GlobalQueue>,

    pub(crate) idle: Idle,

    steal_order: RandomOrder,
}

impl Shard {
    pub(crate) fn new(others: Vec<Other>) -> Shard {
        let count = others.len();
        Self {
            others: others.into_boxed_slice(),
            queue: Arc::new(GlobalQueue::new()),
            idle: Idle::new(),
            steal_order: RandomOrder::new(count),
        }
    }

    pub(crate) fn unpark_one(&self) {
        if let Some(i) = self.idle.processor_to_notify() {
            self.others[i].unparker.unpark();
        }
    }
}

pub(crate) struct Other {
    queue: Arc<LocalQueue>,
    unparker: Unparker,
}

impl Other {
    pub(crate) fn new(queue: Arc<LocalQueue>, unparker: Unparker) -> Self {
        Self {
            queue: queue,
            unparker: unparker,
        }
    }
}

// Processor 负责任务调度和执行
pub(crate) struct Processor {
    index: usize,

    // 共享数据
    shard: Arc<Shard>,

    pub(crate) local: Local,
}

pub(crate) struct Local {
    // 本地数据
    pub(crate) queue: Arc<LocalQueue>,

    parker: Parker,

    fast_rand: FastRand,
}

impl Local {
    pub fn new(queue: Arc<LocalQueue>, parker: Parker) -> Self {
        Self {
            queue: queue,
            parker: parker,
            fast_rand: FastRand::new(seed()),
        }
    }
}

impl Processor {
    pub fn new(index: usize, shard: Arc<Shard>, local: Local) -> Self {
        Self {
            index: index,
            shard: shard,
            local: local,
        }
    }

    pub(crate) fn push(&self, t: Coroutine) {
        let _ = self.local.queue.push(t);
    }

    //调度
    pub(crate) fn schedule(&self) {
        //本地队列
        while let Some(t) = self.local.queue.pop() {
            t.run();
        }
        // steal 偷
        // 从全局队列偷 从其他队列偷
        if let Some(t) = self
            .steal_from_global()
            .or_else(|| self.steal_from_others())
        {
            t.run();
            return;
        }
        //阻塞
        self.park();
    }

    pub(crate) fn steal_from_global(&self) -> Option<Coroutine> {
        // n =  min(len(GQ) / GOMAXPROCS +  1,  cap(LQ) / 2 ) 偷取公式
        // GQ：全局队列总长度（队列中现在元素的个数）
        // GOMAXPROCS：p的个数
        // 至少从全局队列取1个g，但每次不要从全局队列移动太多的g到p本地队列，给其他p留点。这是从全局队列到P本地队列的负载均衡
        let n = cmp::min(
            self.shard.queue.len() / self.shard.others.len() + 1,
            LQ_HALF_SIZE,
        );
        if n == 0 {
            return None;
        }
        for _ in 0..3 {
            let t = self
                .shard
                .queue
                .steal_batch_with_limit_and_pop(self.local.queue.get_ref(), n);
            match t {
                Steal::Empty => return None,
                Steal::Success(t1) => {
                    return Some(t1);
                }
                Steal::Retry => continue,
            }
        }
        None
    }

    pub(crate) fn steal_from_others(&self) -> Option<Coroutine> {
        for i in self
            .shard
            .steal_order
            .start(self.local.fast_rand.fastrand() as usize)
        {
            if i == self.index {
                continue;
            }
            //从其他有G的P哪里偷取一半G过来，放到自己的P本地队列
            let stealer = self.shard.others[i].queue.stealer();
            let n = cmp::max(1, stealer.len() / 2);
            if n == 0 {
                continue;
            }
            let t = stealer.steal_batch_with_limit_and_pop(self.local.queue.get_ref(), n);
            match t {
                Steal::Empty => continue,
                Steal::Success(t1) => {
                    //   println!("steal other run in : p{}", i);
                    return Some(t1);
                }
                Steal::Retry => continue,
            }
        }
        None
    }

    pub(crate) fn unpark_one(&self) {
        if let Some(i) = self.shard.idle.processor_to_notify() {
            self.shard.others[i].unparker.unpark();
        }
    }

    pub(crate) fn park(&self) {
        self.shard.idle.processor_to_parked(self.index);
        self.local.parker.park();
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_processor() {
        //let mut p = Processor::new();
        // p.block_on(serve);
    }

    async fn serve() {
        print!("aaa");
    }
    use std::thread;
    use std::time::Duration;
    #[test]
    fn test_park() {
        let park = Arc::new(Parker::new());
        let park1 = park.clone();
        let t = thread::spawn(move || {
            //thread::sleep(Duration::from_secs(2));
            // park1.park();
            println!("bbbbb");
        });

        thread::sleep(Duration::from_secs(2));

        // park.unpark();
        // park.unpark();
        println!("aaa");
        //thread::sleep(Duration::from_secs(1));
        t.join().unwrap();
    }
}
