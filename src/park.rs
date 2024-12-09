// pub(crate) use parking_lot::{Condvar, Mutex};

// use std::sync::atomic::AtomicUsize;
// use std::{
//     sync::Arc,
//     task::{Context, RawWaker, RawWakerVTable, Waker},
// };

// use std::thread;

// const EMPTY: usize = 0;
// const PARKED_CONDVAR: usize = 1;
// const PARKED_DRIVER: usize = 2;
// const NOTIFIED: usize = 3;
// use std::sync::atomic::Ordering::SeqCst;

// struct Action {
//     state: AtomicUsize,

//     mutex: Mutex<()>,

//     condvar: Condvar,
// }

// impl Action {
//     fn new() -> Self {
//         Self {
//             state: AtomicUsize::new(EMPTY),
//             mutex: Mutex::new(()),
//             condvar: Condvar::new(),
//         }
//     }

//     fn park(&self) {
//         for _ in 0..3 {
//             if self
//                 .state
//                 .compare_exchange(NOTIFIED, EMPTY, SeqCst, SeqCst)
//                 .is_ok()
//             {
//                 return;
//             }
//             thread::yield_now();
//         }
//         let mut m = self.mutex.lock();
//         match self
//             .state
//             .compare_exchange(EMPTY, PARKED_CONDVAR, SeqCst, SeqCst)
//         {
//             Ok(_) => {}
//             Err(NOTIFIED) => {
//                 let old = self.state.swap(EMPTY, SeqCst);
//                 debug_assert_eq!(old, NOTIFIED, "park state changed unexpectedly");

//                 return;
//             }
//             Err(actual) => panic!("inconsistent park state; actual = {}", actual),
//         }

//         loop {
//             self.condvar.wait(&mut m);

//             if self
//                 .state
//                 .compare_exchange(NOTIFIED, EMPTY, SeqCst, SeqCst)
//                 .is_ok()
//             {
//                 return;
//             }
//         }
//     }

//     fn unpark(&self) {
//         match self.state.swap(NOTIFIED, SeqCst) {
//             EMPTY => {}    // no one was waiting
//             NOTIFIED => {} // already unparked
//             PARKED_CONDVAR => {
//                 drop(self.mutex.lock());
//                 self.condvar.notify_one();
//             }
//             actual => panic!("inconsistent state in unpark; actual = {}", actual),
//         }
//     }
// }

// pub(crate) struct Parker(Arc<Action>);

// impl Parker {
//     pub(crate) fn new() -> Parker {
//         Self(Arc::new(Action::new()))
//     }

//     pub(crate) fn unpark(&self) -> UnParker {
//         UnParker(self.0.clone())
//     }

//     pub(crate) fn clone(&self) -> Parker {
//         Parker(self.0.clone())
//     }

//     pub(crate) fn park(&self) {
//         self.0.park()
//     }
// }

// pub(crate) struct UnParker(Arc<Action>);

// impl UnParker {
//     fn new(action: Arc<Action>) -> UnParker {
//         Self(action)
//     }

//     pub(crate) fn unpark(&self) {
//         self.0.unpark()
//     }
// }
