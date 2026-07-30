use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

/// A future that completes after a specified duration.
///
/// `Delay` is used by the async compatibility layer to implement timeout
/// functionality for `block_on` calls.
pub struct Delay {
    deadline: Instant,
}

impl Delay {
    /// Create a new `Delay` that completes after the given duration.
    pub fn new(duration: Duration) -> Self {
        Self {
            deadline: Instant::now() + duration,
        }
    }

    /// Returns the remaining time until the delay completes.
    pub fn remaining(&self) -> Duration {
        let remaining = self.deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            Duration::from_secs(0)
        } else {
            remaining
        }
    }

    /// Returns whether the delay has completed.
    pub fn is_elapsed(&self) -> bool {
        Instant::now() >= self.deadline
    }
}

impl Future for Delay {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<()> {
        if Instant::now() >= self.deadline {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

/// A timeout error returned when a future does not complete within the specified duration.
#[derive(Debug)]
pub struct TimeoutError;

impl std::fmt::Display for TimeoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Async operation timed out")
    }
}

impl std::error::Error for TimeoutError {}