use may::coroutine::{self, Coroutine};
use std::future::Future;
use std::pin::pin;
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

/// VTable for the custom RawWaker.
/// This table defines how to clone, wake, and drop the custom waker which
/// interacts directly with the May coroutine scheduler.
// SAFETY:
// - ptr is valid *const Coroutine from block_on.
// - Unsafe for pointer deref to call unpark.
// - Can't fail: validity ensured by usage, null checked.
static VTABLE: RawWakerVTable = RawWakerVTable::new(
    |ptr| RawWaker::new(ptr, &VTABLE),
    |ptr| unsafe {
        if ptr.is_null() {
            return;
        }
        (&*(ptr as *const Coroutine)).unpark()
    },
    |ptr| unsafe {
        if ptr.is_null() {
            return;
        }
        (&*(ptr as *const Coroutine)).unpark()
    },
    |_| {},
);

/// Blocks the current May coroutine until the given future is resolved.
///
/// This is the core executor for Feather's async support. It allows stackful coroutines
/// to wait on standard Rust `Future`s without blocking the underlying OS thread.
///
/// This must be called from within a May coroutine context.
pub fn block_on<F: Future>(fut: F) -> F::Output {
    let mut fut = pin!(fut);
    // Safe pinning via pin!.
    let current_co = std::panic::catch_unwind(|| coroutine::current()).unwrap_or_else(|_| panic!("block_on must be called from within a May coroutine context."));
    // Coroutine context checked.

    let raw_waker = RawWaker::new(&current_co as *const _ as *const (), &VTABLE);
    let waker = unsafe { Waker::from_raw(raw_waker) };
    // SAFETY:
    // - data points to live Coroutine.
    // - VTable provides correct wake behavior.
    // - Can't fail: coroutine context verified.
    let mut cx = Context::from_waker(&waker);

    loop {
        match fut.as_mut().poll(&mut cx) {
            Poll::Ready(res) => return res,
            Poll::Pending => coroutine::park(),
        }
    }
}
