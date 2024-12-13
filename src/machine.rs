use std::sync::Arc;

use std::thread;

use crate::processor;
use crate::processor::Processor;

//线程池实现
pub(crate) struct ThreadPool;

impl ThreadPool {
    pub(crate) fn launch(ps: &mut Vec<Arc<Processor>>) {
        for (i, p) in ps.drain(..(ps.len() - 1)).enumerate() {
            thread::spawn(|| processor::run(p));
        }
    }

    pub(crate) fn execute() {}
}
