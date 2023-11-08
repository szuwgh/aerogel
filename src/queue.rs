use crate::deque::Injector;
use crate::deque::Steal;
use crate::deque::Stealer;
use crate::deque::Worker;
use crate::task::Schedule;
const LQ_SIZE: usize = 256;
pub(crate) const LQ_HALF_SIZE: usize = LQ_SIZE / 2;
use crate::task::Task;

use crate::processor::EX;

pub(crate) type Coroutine = Task<LocalScheduler>;
pub(crate) struct LocalScheduler;

impl Schedule for LocalScheduler {
    fn schedule(&self, task: Coroutine) {
        EX.with(|ex| {
            ex.0.push(task);
            ex.0.unpark_one();
        });
    }

    fn yield_now(&self, task: Coroutine) {
        self.schedule(task)
    }
}

pub(crate) struct LocalQueue {
    queue: Worker<Coroutine>,
}

impl LocalQueue {
    pub(crate) fn new() -> LocalQueue {
        Self {
            queue: Worker::new_fifo(LQ_SIZE),
        }
    }
    pub(crate) fn pop(&self) -> Option<Coroutine> {
        self.queue.pop()
    }

    pub(crate) fn len(&self) -> usize {
        self.queue.len()
    }

    pub(crate) fn push(&self, t: Coroutine) -> Result<(), Coroutine> {
        self.queue.push(t)
    }

    pub(crate) fn stealer(&self) -> Stealer<Coroutine> {
        self.queue.stealer()
    }

    pub(crate) fn get_ref(&self) -> &Worker<Coroutine> {
        &self.queue
    }
}

pub(crate) struct GlobalQueue {
    queue: Injector<Coroutine>,
}

impl GlobalQueue {
    pub(crate) fn new() -> GlobalQueue {
        Self {
            queue: Injector::new(),
        }
    }

    pub(crate) fn push(&self, t: Coroutine) {
        self.queue.push(t)
    }

    pub(crate) fn len(&self) -> usize {
        self.queue.len()
    }

    pub(crate) fn get_ref(&self) -> &Injector<Coroutine> {
        &self.queue
    }

    pub(crate) fn steal_batch_with_limit_and_pop(
        &self,
        dest: &Worker<Coroutine>,
        limit: usize,
    ) -> Steal<Coroutine> {
        self.queue.steal_batch_with_limit_and_pop(dest, limit)
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_localqueue() {}

    #[test]
    fn test_globalqueue() {}
}
