use parking_lot::Condvar;
use parking_lot::Mutex;
use std::fmt::Debug;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
pub(crate) enum JobResult<T> {
    None,
    Ok(T),
}

pub(crate) struct Job {
    func: unsafe fn(data: *const ()),
    data: *const (),
}

impl Job {
    pub(crate) fn run(&self) {
        unsafe { (self.func)(self.data) }
    }
}

pub struct StackJob<F, R>
where
    F: FnOnce() -> R,
{
    f: Option<F>,
    pub lock: SpinLatch,
    result: JobResult<R>,
}

impl<F, R> StackJob<F, R>
where
    F: FnOnce() -> R,
{
    pub(crate) fn new(f: F) -> StackJob<F, R> {
        Self {
            f: Some(f),
            lock: SpinLatch::new(),
            result: JobResult::None,
        }
    }

    pub(crate) fn get_res(self) -> R {
        match self.result {
            JobResult::None => panic!("job function panicked"),
            JobResult::Ok(x) => x,
        }
    }

    pub(crate) fn as_job(&self) -> Job {
        unsafe fn execute<F, R>(data: *const ())
        where
            F: FnOnce() -> R,
        {
            let job = data as *mut StackJob<F, R>;
            struct PanicGuard<'a>(&'a SpinLatch);
            impl<'a> Drop for PanicGuard<'a> {
                fn drop(&mut self) {
                    self.0.done();
                }
            }
            let _guard = PanicGuard(&(*job).lock);
            let func = (*job).f.take().unwrap();
            let res = func();
            (*job).result = JobResult::Ok(res);
        }

        Job {
            func: execute::<F, R>,
            data: self as *const _ as *const (),
        }
    }
}

pub struct JobHandle<F, R>(pub(crate) Box<StackJob<F, R>>)
where
    F: FnOnce() -> R;

impl<F, R> JobHandle<F, R>
where
    F: FnOnce() -> R,
{
    pub(crate) fn new(f: F) -> JobHandle<F, R> {
        Self(Box::new(StackJob::new(f)))
    }

    pub(crate) fn as_job(&mut self) -> Job {
        self.0.as_job()
    }

    pub fn get_res(self) -> R {
        self.0.lock.wait();
        self.0.get_res()
    }
}

pub struct SpinLatch {
    b: AtomicBool,
}

impl SpinLatch {
    #[inline]
    pub fn new() -> SpinLatch {
        SpinLatch {
            b: AtomicBool::new(false),
        }
    }

    /// Test if latch is set.
    #[inline]
    pub fn wait(&self) -> bool {
        !self.b.load(Ordering::Acquire)
    }

    #[inline]
    fn done(&self) {
        self.b.store(true, Ordering::Release);
    }
}

pub struct LockLatch {
    m: Mutex<bool>,
    v: Condvar,
}

impl LockLatch {
    #[inline]
    pub fn new() -> LockLatch {
        LockLatch {
            m: Mutex::new(false),
            v: Condvar::new(),
        }
    }

    #[inline]
    pub fn wait(&self) {
        let mut guard = self.m.lock();
        while !*guard {
            self.v.wait(&mut guard);
        }
    }

    #[inline]
    fn done(&self) {
        let mut guard = self.m.lock();
        *guard = true;
        self.v.notify_all();
    }
}

mod tests {
    use super::*;
    use core::time;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;
    #[test]
    fn test_latch_lock() {
        let lock = Arc::new(LockLatch::new());
        let lock1 = lock.clone();
        thread::spawn(move || {
            lock1.wait();
            println!("aa");
        });
        lock.done();
        println!("xx");
        thread::sleep(Duration::from_secs(2));
    }
}
