use crate::processor::EX;
use crate::{park::UnParker, task::Task};
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

// //协程实现
// pub(crate) struct Routine {
//     future: RefCell<LocalBoxFuture<'static, ()>>,
// }

// impl Routine {
//     pub(crate) fn new(future: RefCell<LocalBoxFuture<'static, ()>>) -> Routine {
//         Self { future: future }
//     }

//     pub(crate) fn run(self: Coroutine) {
//         let w = get_waker(self.clone());
//         let mut context = Context::from_waker(&w);
//         let _ = Pin::new(self.future.borrow_mut())
//             .as_mut()
//             .poll(&mut context);
//     }

//     fn wake_by_ref(self: &Coroutine) {
//         EX.with(|ex| {
//             ex.0.push(self.clone());
//             ex.0.unpark_one()
//         });
//     }

//     fn wake(self: Coroutine) {
//         self.wake_by_ref()
//     }
// }

// fn get_waker(co: Coroutine) -> Waker {
//     let raw_waker = RawWaker::new(Arc::into_raw(co) as *const (), &WakerImpl::VTABLE);
//     unsafe { Waker::from_raw(raw_waker) }
// }

// struct WakerImpl;

// impl WakerImpl {
//     const VTABLE: RawWakerVTable = RawWakerVTable::new(
//         Self::clone_waker,
//         Self::wake,
//         Self::wake_by_ref,
//         Self::drop_waker,
//     );

//     unsafe fn clone_waker(data: *const ()) -> RawWaker {
//         let vtable = &Self::VTABLE;
//         RawWaker::new(data, vtable)
//     }

//     unsafe fn wake(ptr: *const ()) {
//         let rc = Arc::from_raw(ptr as *const Routine);
//         rc.wake();
//     }

//     unsafe fn wake_by_ref(ptr: *const ()) {
//         let rc = mem::ManuallyDrop::new(Arc::from_raw(ptr as *const Routine));
//         rc.wake_by_ref();
//     }

//     unsafe fn drop_waker(ptr: *const ()) {
//         drop(Rc::from_raw(ptr as *const Coroutine));
//     }
// }
