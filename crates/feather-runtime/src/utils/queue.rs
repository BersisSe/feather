use parking_lot::{Condvar, Mutex};
use std::collections::VecDeque;
/// A simple enum to represent the Flow-State of the queue
/// This enum is used to differentiate between a value and an unblock signal.
/// The `Value` variant contains the actual value, while the `Unblock` variant
pub enum QueueFlow<T> {
    Value(T),
    Unblock,
}

/// A thread-safe queue that allows blocking and unblocking operations.
/// This queue is designed to be used in a multi-threaded environment.
pub struct Queue<T> {
    /// A mutex to protect access to the queue
    /// This ensures that only one thread can access the queue at a time.
    queue: Mutex<VecDeque<QueueFlow<T>>>,
    /// A condition variable to notify waiting threads
    condvar: Condvar,
}

impl<T> Queue<T> {
    /// Creates a new `Queue` instance with the specified capacity.
    /// The capacity is the maximum number of items that can be stored in the queue.
    /// The queue will grow dynamically if more items are added.
    /// The `size` parameter specifies the initial capacity of the queue.
    pub fn with_capacity(size: usize) -> Self {
        Queue {
            queue: Mutex::new(VecDeque::with_capacity(size)),
            condvar: Condvar::new(),
        }
    }
    /// Pushes new item into to back of the queue.
    /// Threads Get notified when a new item is pushed via Condvar.
    pub fn push(&self, item: T) {
        let mut queue = self.queue.lock();
        queue.push_back(QueueFlow::Value(item));
        self.condvar.notify_one();
    }
    ///Blocks Until a value is available to pop
    /// if Unblock is called it will return None
    pub fn pop(&self) -> Option<T> {
        let mut queue = self.queue.lock();
        loop {
            match queue.pop_front() {
                Some(QueueFlow::Value(v)) => return Some(v),
                Some(QueueFlow::Unblock) => return None,
                None => (),
            }
            self.condvar.wait(&mut queue);
        }
    }
    /// Unblocks the queue, all the threads that are struck in the pop method will return None
    pub fn unblock(&self) {
        let mut queue = self.queue.lock();
        queue.push_back(QueueFlow::Unblock);
        self.condvar.notify_one();
    }
}
