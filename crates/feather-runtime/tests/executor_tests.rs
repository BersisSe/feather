use std::future::Future;
use std::sync::Arc;
use std::task::{Context, RawWaker, RawWakerVTable, Waker};
use std::time::{Duration, Instant};

use may::coroutine::{self, Coroutine};

use feather_runtime::runtime::executor::{block_on, block_on_with_timeout};
use feather_runtime::runtime::timeout::TimeoutError;

fn make_waker() -> Waker {
    static VTABLE: RawWakerVTable = RawWakerVTable::new(
        |ptr| {
            let arc = unsafe { Arc::from_raw(ptr as *const Coroutine) };
            let cloned = Arc::clone(&arc);
            RawWaker::new(Arc::into_raw(cloned) as *const (), &VTABLE)
        },
        |ptr| unsafe {
            let arc = Arc::from_raw(ptr as *const Coroutine);
            arc.unpark();
        },
        |ptr| unsafe {
            let arc = Arc::from_raw(ptr as *const Coroutine);
            arc.unpark();
            std::mem::forget(arc);
        },
        |ptr| unsafe {
            drop(Arc::from_raw(ptr as *const Coroutine));
        },
    );

    let current_co = Arc::new(coroutine::current());
    let raw_waker = RawWaker::new(Arc::into_raw(current_co) as *const (), &VTABLE);
    unsafe { Waker::from_raw(raw_waker) }
}

#[test]
fn test_block_on_with_coroutine() {
    may::config().set_workers(1);
    may::config().set_stack_size(64 * 1024);

    may::go!(|| {
        let result = block_on(std::future::ready("hello from coroutine"));
        assert_eq!(result, "hello from coroutine");
    });
}

#[test]
fn test_block_on_with_delayed_future() {
    may::config().set_workers(1);
    may::config().set_stack_size(64 * 1024);

    may::go!(|| {
        let _start = Instant::now();
        let delay = Duration::from_millis(100);
        let result = block_on(async move {
            let fut = std::future::pending::<()>();
            let _ = std::pin::pin!(fut).as_mut().poll(&mut Context::from_waker(&make_waker()));
            std::thread::sleep(delay);
            "delayed"
        });
        assert_eq!(result, "delayed");
    });
}

#[test]
fn test_block_on_with_timeout_success() {
    may::config().set_workers(1);
    may::config().set_stack_size(64 * 1024);

    may::go!(|| {
        let result = block_on_with_timeout(std::future::ready("success"), Some(Duration::from_secs(5)));
        assert_eq!(result.unwrap(), "success");
    });
}

#[test]
fn test_block_on_with_timeout_exceeded() {
    may::config().set_workers(1);
    may::config().set_stack_size(64 * 1024);

    may::go!(|| {
        let start = Instant::now();
        let result = block_on_with_timeout(async {
            std::thread::sleep(Duration::from_secs(10));
            "should not reach"
        }, Some(Duration::from_millis(50)));

        let elapsed = start.elapsed();
        assert!(result.is_err());
        assert!(elapsed >= Duration::from_millis(50));
        assert!(elapsed < Duration::from_millis(200));
    });
}

#[test]
fn test_block_on_with_timeout_no_timeout_passes() {
    may::config().set_workers(1);
    may::config().set_stack_size(64 * 1024);

    may::go!(|| {
        let result = block_on_with_timeout(std::future::ready(42), None);
        assert_eq!(result.unwrap(), 42);
    });
}
