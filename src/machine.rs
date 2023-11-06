use std::cell::Ref;
use std::sync::{Arc, RwLock};

use std::{cell::RefCell, num::NonZeroU64, thread};

use crate::processor;
use crate::processor::Processor;

//线程池实现
pub(crate) struct ThreadPool;

impl ThreadPool {
    pub(crate) fn launch(ps: &mut Vec<Arc<Processor>>) {
        for p in ps.drain(..(ps.len() - 1)) {
            thread::spawn(|| processor::run(p));
        }
    }
}

pub(crate) struct Machine {
    id: usize,
}

impl Machine {
    fn run<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        thread::spawn(|| {
            f();
        });
    }
}
