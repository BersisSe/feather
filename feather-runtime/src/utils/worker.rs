// src/utils/worker.rs
use crossbeam::channel::{unbounded, Receiver, Sender};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

const MIN_WORKER: usize = 6;
const MAX_WORKER: usize = 60;
const IDLE_TIMEOUT: Duration = Duration::from_secs(30);

pub struct TaskPool {
    sender: Sender<Job>,
    receiver: Receiver<Job>,
    active_workers: AtomicUsize,
    idle_workers: Arc<AtomicUsize>,
}

impl TaskPool {
    pub fn new() -> Self {
        let (sender, receiver) = unbounded();
        let pool = TaskPool {
            sender,
            receiver,
            active_workers: AtomicUsize::new(0),
            idle_workers: Arc::new(AtomicUsize::new(0)),
        };

        for _ in 0..MIN_WORKER {
            pool.add_worker();
        }

        pool
    }

    pub fn add_task(&self, task: Job) {
        self.sender.send(task).unwrap();

        // Dynamically add workers if needed
        if self.idle_workers.load(Ordering::Acquire) == 0
            && self.active_workers.load(Ordering::Acquire) < MAX_WORKER
        {
            self.add_worker();
        }
    }

    fn add_worker(&self) {
        self.active_workers.fetch_add(1, Ordering::Release);
        let idle_workers = self.idle_workers.clone();
        let receiver = self.receiver.clone();
        thread::spawn(move || {
            loop {
                match receiver.recv_timeout(IDLE_TIMEOUT) {
                    Ok(job) => match job {
                        Job::Task(task) => {
                            idle_workers.fetch_sub(1, Ordering::Release);
                            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(task));
                            idle_workers.fetch_add(1, Ordering::Release);
                        }
                        Job::Stop => break,
                    },
                    Err(_) => {
                        // Timeout: terminate the worker if idle
                        if idle_workers.fetch_sub(1, Ordering::Release) == 1 {
                            break;
                        }
                    }
                }
            }
        });
    }
}

impl Drop for TaskPool {
    fn drop(&mut self) {
        for _ in 0..self.active_workers.load(Ordering::Acquire) {
            self.sender.send(Job::Stop).unwrap();
        }
    }
}

/// Represents a job to be executed by the worker pool.
/// It can be either a task (a closure) or a signal to stop the worker.
pub enum Job {
    Task(Box<dyn FnOnce() + Send + 'static>),
    Stop,
}

