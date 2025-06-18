use crossbeam::channel::{Receiver, Sender, unbounded};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;

#[derive(Debug)]
pub struct PoolConfig {
    pub max_workers: usize,
    pub min_workers: usize,
    pub timeout: Duration,
}
impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_workers: 60,
            min_workers: 6,
            timeout: Duration::from_secs(30),
        }
    }
}

/// A Dynamic TaskPool
/// If there is not enough threads for the task it creates new ones
pub struct TaskPool {
    sender: Sender<Job>,
    receiver: Receiver<Job>,
    active_workers: AtomicUsize,
    idle_workers: Arc<AtomicUsize>,
    config: PoolConfig,
}

impl TaskPool {
    /// Create a new TaskPool with the default configration
    pub fn new() -> Self {
        let config = PoolConfig::default();
        let min_workers = config.min_workers;
        let (sender, receiver) = unbounded();
        let pool = TaskPool {
            sender,
            receiver,
            active_workers: AtomicUsize::new(0),
            idle_workers: Arc::new(AtomicUsize::new(0)),
            config,
        };

        for _ in 0..min_workers {
            pool.add_worker();
        }

        pool
    }
    /// Create a new TaskPool with the given configration
    pub fn with_config(config: PoolConfig) -> Self {
        let min_workers = config.min_workers;
        let (sender, receiver) = unbounded();
        let pool = TaskPool {
            sender,
            receiver,
            active_workers: AtomicUsize::new(0),
            idle_workers: Arc::new(AtomicUsize::new(0)),
            config,
        };

        for _ in 0..min_workers {
            pool.add_worker();
        }

        pool
    }
    /// Add's new task to the sender channel of the TaskPool
    pub fn add_task(&self, task: Job) {
        self.sender.send(task).unwrap();

        if self.idle_workers.load(Ordering::Acquire) == 0 && self.active_workers.load(Ordering::Acquire) < self.config.max_workers {
            self.add_worker();
        }
    }
    /// Add's a new worker if the load is higher than task pool can handle
    fn add_worker(&self) {
        self.active_workers.fetch_add(1, Ordering::Release);
        let idle_workers = self.idle_workers.clone();
        let receiver = self.receiver.clone();
        let timeout = self.config.timeout;
        thread::spawn(move || {
            loop {
                match receiver.recv_timeout(timeout) {
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
/// Dropin here babyyyyy
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

impl Into<Job> for Box<dyn FnOnce() + Send + 'static> {
    fn into(self) -> Job {
        Job::Task(self)
    }
}
