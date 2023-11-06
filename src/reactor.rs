use futures::task::Waker;
use polling::Events;
use polling::Poller;
use std::collections::HashMap;

// Reactor 负责唤醒对应 Coroutine
pub(crate) struct Reactor {
    poller: Poller,
    waker_mapping: HashMap<u64, Waker>,
    buffer: Events,
}

impl Reactor {
    pub fn wait(&mut self) {
        println!("[reactor] waiting");
        self.poller.wait(&mut self.buffer, None);
        println!("[reactor] wait done");

        for event in self.buffer.iter() {
            // let event = self.buffer.swap_remove(0);
            if event.readable {
                if let Some(waker) = self.waker_mapping.remove(&(event.key as u64 * 2)) {
                    println!(
                        "[reactor token] fd {} read waker token {} removed and woken",
                        event.key,
                        event.key * 2
                    );
                    waker.wake();
                }
            }
            if event.writable {
                if let Some(waker) = self.waker_mapping.remove(&(event.key as u64 * 2 + 1)) {
                    println!(
                        "[reactor token] fd {} write waker token {} removed and woken",
                        event.key,
                        event.key * 2 + 1
                    );
                    waker.wake();
                }
            }
        }
    }
}
