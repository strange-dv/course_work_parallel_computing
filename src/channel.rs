use std::collections::VecDeque;
use std::sync::{Condvar, Mutex};

pub struct Channel<T> {
    queue: Mutex<VecDeque<T>>,
    items: Condvar,
}

impl<T> Channel<T> {
    pub fn new() -> Self {
        Channel {
            queue: Mutex::new(VecDeque::new()),
            items: Condvar::new(),
        }
    }

    pub fn send(&self, t: T) {
        let mut queue = self.queue.lock().unwrap();
        queue.push_back(t);
        self.items.notify_one();
    }

    pub fn receive(&self) -> Option<T> {
        let mut queue = self.queue.lock().unwrap();
        loop {
            if let Some(t) = queue.pop_front() {
                return Some(t);
            }
            queue = self.items.wait(queue).unwrap();
        }
    }
}
