use std::cell::Ref;
use std::sync::{Arc, RwLock};

use std::thread;

use crate::processor;
use crate::processor::Processor;

//线程池实现
pub(crate) struct ThreadPool;

impl ThreadPool {
    pub(crate) fn launch(ps: &mut Vec<Arc<Processor>>) {
        for (i, p) in ps.drain(..(ps.len() - 1)).enumerate() {
            println!("{}", i);
            thread::spawn(|| processor::run(p));
        }
    }
}
