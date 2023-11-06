use crate::park::UnParker;
use crate::processor::EX;
use futures::future::LocalBoxFuture;
use std::cell::RefCell;
use std::rc::Rc;

use futures::future::Future;
use std::{
    mem,
    pin::Pin,
    sync::Arc,
    task::{Context, RawWaker, RawWakerVTable, Waker},
};

pub(crate) type Coroutine = Rc<Routine>;

//协程实现
pub(crate) struct Routine {
    future: RefCell<LocalBoxFuture<'static, ()>>,
    parker: UnParker,
}

impl Routine {
    pub(crate) fn new(future: RefCell<LocalBoxFuture<'static, ()>>, parker: UnParker) -> Routine {
        Self {
            future: future,
            parker: parker,
        }
    }

    pub(crate) fn run(self: Coroutine) {
        let future = self.future.borrow_mut();
        let w = get_waker(self.clone());
        let mut context = Context::from_waker(&w);
        let _ = Pin::new(future).as_mut().poll(&mut context);
    }

    fn wake_by_ref(self: &Rc<Self>) {
        EX.with(|ex| ex.0.push(self.clone()));
        self.parker.unpark()
    }

    fn wake(self: Rc<Self>) {
        self.wake_by_ref()
    }
}

fn get_waker(co: Coroutine) -> Waker {
    let raw_waker = RawWaker::new(Rc::into_raw(co) as *const (), &WakerImpl::VTABLE);
    unsafe { Waker::from_raw(raw_waker) }
}

struct WakerImpl;

impl WakerImpl {
    const VTABLE: RawWakerVTable = RawWakerVTable::new(
        Self::clone_waker,
        Self::wake,
        Self::wake_by_ref,
        Self::drop_waker,
    );

    unsafe fn clone_waker(data: *const ()) -> RawWaker {
        let vtable = &Self::VTABLE;
        RawWaker::new(data, vtable)
    }

    unsafe fn wake(ptr: *const ()) {
        let rc = Rc::from_raw(ptr as *const Routine);
        rc.wake();
    }

    unsafe fn wake_by_ref(ptr: *const ()) {
        let rc = mem::ManuallyDrop::new(Rc::from_raw(ptr as *const Routine));
        rc.wake_by_ref();
    }

    unsafe fn drop_waker(ptr: *const ()) {
        drop(Rc::from_raw(ptr as *const Coroutine));
    }
}
