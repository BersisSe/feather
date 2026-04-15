use may::coroutine::{self, Coroutine};
use std::future::Future;
use std::pin::pin;
use std::sync::Arc;
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

/// VTable for the custom RawWaker.
///
/// Each entry receives a `*const ()` that is actually an `Arc<Coroutine>`
/// that has been converted via `Arc::into_raw`. Ownership rules:
///
/// - `clone`      - borrows the Arc, increments refcount, returns a new owning pointer.
/// - `wake`       - takes ownership of the Arc (consumes the pointer), unparks, then drops.
/// - `wake_by_ref`- borrows the Arc (does NOT consume the pointer), unparks, then forgets.
/// - `drop`       - takes ownership of the Arc and drops it, decrementing the refcount.
static VTABLE: RawWakerVTable = RawWakerVTable::new(
    // clone: increment Arc refcount and return a new owning raw pointer
    |ptr| {
        let arc = unsafe { Arc::from_raw(ptr as *const Coroutine) };
        let cloned = Arc::clone(&arc);
        // Forget the original so we don't decrement its refcount —
        // the caller still owns that pointer.
        std::mem::forget(arc);
        RawWaker::new(Arc::into_raw(cloned) as *const (), &VTABLE)
    },
    // wake (by value): owns the pointer - unpark then drop the Arc
    |ptr| unsafe {
        let arc = Arc::from_raw(ptr as *const Coroutine);
        arc.unpark();
        // arc drops here, decrementing the refcount
    },
    // wake_by_ref: borrows the pointer - unpark without dropping
    |ptr| unsafe {
        let arc = Arc::from_raw(ptr as *const Coroutine);
        arc.unpark();
        // forget so we don't decrement the refcount - caller still owns it
        std::mem::forget(arc);
    },
    // drop: takes ownership of the pointer and drops the Arc
    |ptr| unsafe {
        drop(Arc::from_raw(ptr as *const Coroutine));
    },
);

/// Blocks the current May coroutine until the given future resolves.
///
/// This is the core executor for Feather's async compatibility layer. It allows
/// stackful May coroutines to drive standard Rust `Future`s without blocking
/// the underlying OS thread when a future returns `Pending`, the coroutine
/// is parked and the OS thread is freed for other work.
///
/// # Panics
///
/// Panics if called from outside a May coroutine context.
pub fn block_on<F: Future>(fut: F) -> F::Output {
    let mut fut = pin!(fut);

    // Verify we are inside a May coroutine and grab a handle to the current one.
    // The handle is heap-allocated via Arc so the waker pointer remains valid
    // even if a future clones and stores the waker beyond this call.
    let current_co = Arc::new(
        std::panic::catch_unwind(|| coroutine::current())
            .unwrap_or_else(|_| panic!("block_on must be called from within a May coroutine context.")),
    );

    // Convert the Arc into a raw pointer — ownership is transferred into the RawWaker.
    // The vtable drop entry is responsible for eventually calling Arc::from_raw to
    // reclaim it.
    let raw_waker = RawWaker::new(Arc::into_raw(current_co) as *const (), &VTABLE);

    // SAFETY:
    // - The pointer is a valid, heap-allocated Arc<Coroutine>.
    // - The vtable correctly handles clone/wake/drop ownership.
    // - The coroutine context is verified above via catch_unwind.
    let waker = unsafe { Waker::from_raw(raw_waker) };
    let mut cx = Context::from_waker(&waker);

    loop {
        match fut.as_mut().poll(&mut cx) {
            Poll::Ready(res) => return res,
            // Future not ready — park the coroutine. The waker will call
            // unpark() when the future's resource is ready, resuming us here.
            Poll::Pending => coroutine::park(),
        }
    }
}