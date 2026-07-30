use may::coroutine::{self, Coroutine};
use std::collections::HashMap;
use std::future::Future;
use std::pin::pin;
use std::sync::Arc;
use std::sync::RwLock;
use std::sync::atomic::{AtomicBool, Ordering};
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
use std::panic::AssertUnwindSafe;
use std::time::{Duration, Instant};

use crate::runtime::timeout::TimeoutError;

type CoroutinePtr = *const Coroutine;
type WakeFlag = Arc<AtomicBool>;

thread_local! {
    static WAKE_FLAGS: RwLock<HashMap<CoroutinePtr, WakeFlag>> = RwLock::new(HashMap::new());
}

fn insert_wake_flag(ptr: CoroutinePtr) -> WakeFlag {
    let flag = Arc::new(AtomicBool::new(true));
    WAKE_FLAGS.with(|flags| {
        flags.write().unwrap().insert(ptr, flag.clone());
    });
    flag
}

fn remove_wake_flag(ptr: CoroutinePtr) {
    WAKE_FLAGS.with(|flags| {
        flags.write().unwrap().remove(&ptr);
    });
}

fn set_woken(ptr: CoroutinePtr) {
    WAKE_FLAGS.with(|flags| {
        if let Some(flag) = flags.read().unwrap().get(&ptr) {
            flag.store(false, Ordering::SeqCst);
        }
    });
}

fn should_park(ptr: CoroutinePtr) -> bool {
    WAKE_FLAGS.with(|flags| {
        flags.read().unwrap().get(&ptr).map_or(true, |flag| {
            flag.load(Ordering::SeqCst)
        })
    })
}

static VTABLE: RawWakerVTable = RawWakerVTable::new(
    |ptr| {
        let co_ptr = ptr as *const Coroutine;
        let arc = unsafe { Arc::from_raw(co_ptr) };
        let cloned = Arc::clone(&arc);
        insert_wake_flag(co_ptr);
        RawWaker::new(Arc::into_raw(cloned) as *const (), &VTABLE)
    },
    |ptr| unsafe {
        let co_ptr = ptr as *const Coroutine;
        set_woken(co_ptr);
        let arc = Arc::from_raw(co_ptr);
        arc.unpark();
    },
    |ptr| unsafe {
        let co_ptr = ptr as *const Coroutine;
        set_woken(co_ptr);
        let arc = Arc::from_raw(co_ptr);
        arc.unpark();
        std::mem::forget(arc);
    },
    |ptr| unsafe {
        let co_ptr = ptr as *const Coroutine;
        remove_wake_flag(co_ptr);
        drop(Arc::from_raw(co_ptr));
    },
);

/// Blocks the current May coroutine until the given future resolves.
///
/// This is the core executor for Feather's async compatibility layer. It allows
/// stackful May coroutines to drive standard Rust `Future`s without blocking
/// the underlying OS thread when a future returns `Pending`, the coroutine
/// is parked and the OS thread is freed for other work.
///
/// For timeout support, see [`block_on_with_timeout`].
///
/// # Panics
///
/// Panics if called from outside a May coroutine context or if the future panics.
pub fn block_on<F: Future>(fut: F) -> F::Output {
    block_on_with_timeout(fut, None)
        .expect("block_on_with_timeout with no timeout cannot fail")
}

/// Blocks the current May coroutine until the given future resolves or a timeout occurs.
///
/// This is the core executor for Feather's async compatibility layer with timeout support.
/// It allows stackful May coroutines to drive standard Rust `Future`s without blocking
/// the underlying OS thread when a future returns `Pending`, the coroutine
/// is parked and the OS thread is freed for other work.
///
/// # Arguments
///
/// * `fut` - The future to execute
/// * `timeout` - Optional timeout duration. If `None`, no timeout is applied.
///
/// # Returns
///
/// * `Ok(output)` - The future completed successfully within the timeout
/// * `Err(TimeoutError)` - The future did not complete within the timeout
///
/// # Panics
///
/// Panics if called from outside a May coroutine context or if the future panics.
pub fn block_on_with_timeout<F: Future>(
    fut: F,
    timeout: Option<Duration>,
) -> Result<F::Output, TimeoutError> {
    let mut fut = pin!(fut);
    
    let current_co = Arc::new(
        std::panic::catch_unwind(|| coroutine::current())
            .unwrap_or_else(|_| panic!("block_on must be called from within a May coroutine context.")),
    );

    let co_ptr = Arc::into_raw(current_co.clone()) as *const Coroutine;
    insert_wake_flag(co_ptr);

    let raw_waker = RawWaker::new(co_ptr as *const (), &VTABLE);
    let waker = unsafe { Waker::from_raw(raw_waker) };
    let mut cx = Context::from_waker(&waker);

    // Set up deadline if timeout is provided
    let deadline = timeout.map(|t| Instant::now() + t);
   
    let result = std::panic::catch_unwind(AssertUnwindSafe(move || {
        loop {
            // Check timeout first
            if let Some(dl) = deadline {
                if Instant::now() >= dl {
                    return Err(TimeoutError);
                }
            }

            match fut.as_mut().poll(&mut cx) {
                Poll::Ready(res) => return Ok(res),
                Poll::Pending => {
                    if should_park(co_ptr) {
                        coroutine::park();
                    } else {
                        set_woken(co_ptr);
                    }
                }
            }
        }
    }));
    
    match result {
        Ok(res) => res,
        Err(panic_info) => std::panic::resume_unwind(panic_info),
    }
}