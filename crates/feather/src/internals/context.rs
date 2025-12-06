use parking_lot::{Mutex, MutexGuard, RwLock};
use std::any::{Any, TypeId};
use std::collections::HashMap;

use std::sync::Arc;

#[cfg(feature = "jwt")]
use crate::jwt::JwtManager;

type Erased = dyn Any + Send + Sync;

/// A simple easier to use Wrapper around parking_lot Mutex can be used for mutable data in the AppContext
/// # Example
/// ```rust,ignore
/// use feather::{AppContext, State};
///
/// #[derive(Debug)]
/// struct Counter {
///     count: i32,
/// }
///
/// impl Counter {
///     fn increment(&mut self) {
///         self.count += 1;
///     }
/// }
///
/// // Store in context
/// app.context().set_state(State::new(Counter { count: 0 }));
///
/// // In a middleware
/// app.get("/", middleware!(|_req, res, ctx| {
///     let mut counter = ctx.get_state::<State<Counter>>();
///     counter.increment(); // Direct method call!
///     res.send_text(format!("Count: {}", counter.count));
///     next!()
/// }));
/// ```
pub struct State<S> {
    inner: Mutex<S>,
}

impl<S> State<S> {
    /// Creates a new State wrapping the given value.
    pub fn new(state: S) -> Self {
        Self {
            inner: Mutex::new(state),
        }
    }

    /// Get a lock guard to access the inner state.
    ///
    /// This is useful when you need to hold the lock for multiple operations.
    ///
    /// # Example
    /// ```rust,ignore
    /// let counter = ctx.get_state::<State<Counter>>();
    /// let mut guard = counter.lock();
    /// guard.count += 1;
    /// guard.count += 1; // Multiple operations with one lock
    /// ```
    pub fn lock(&self) -> MutexGuard<'_, S> {
        self.inner.lock()
    }

    /// Execute a closure with access to the inner state.
    ///
    /// # Example
    /// ```rust,ignore
    /// let counter = ctx.get_state::<State<Counter>>();
    /// counter.with(|c| {
    ///    println!("Current count: {}", c.count);
    /// });
    /// ```
    /// **DO NOT USE THE STATE ITSELF IN THE SCOPE IT WILL COUSE A DEADLOCK!**
    pub fn with_scope<R>(&self, f: impl FnOnce(&S) -> R) -> R {
        let guard = self.inner.lock();
        f(&guard)
    }

    /// This is the most ergonomic way to work with State.
    /// # Example
    /// ```rust,ignore
    /// let counter = ctx.get_state::<State<Counter>>();
    /// counter.with_mut(|c| {
    ///     c.increment();
    ///     c.increment();
    /// });
    /// ```
    /// **DO NOT USE THE STATE ITSELF IN THE SCOPE IT WILL COUSE A DEADLOCK!**
    pub fn with_mut_scope<R>(&self, f: impl FnOnce(&mut S) -> R) -> R {
        let mut guard = self.inner.lock();
        f(&mut guard)
    }
}

// Make State cloneable if the inner type is cloneable
impl<S: Clone> State<S> {
    /// Get a clone of the inner state.
    pub fn get_clone(&self) -> S {
        self.inner.lock().clone()
    }
}

#[derive(Clone)]
pub struct AppContext {
    pub inner: Arc<RwLock<HashMap<TypeId, Arc<Erased>>>>,
    #[cfg(feature = "jwt")]
    jwt: Option<JwtManager>,
}

impl AppContext {
    /// Create an empty AppContext
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
            #[cfg(feature = "jwt")]
            jwt: None,
        }
    }
    /// Sets the internal [jwt] Field with a given Manager
    #[cfg(feature = "jwt")]
    pub fn set_jwt(&mut self, jwt: JwtManager) {
        if self.jwt.is_none() {
            self.jwt = Some(jwt)
        }
    }
    /// Used to Access the JwtManager inside.  
    /// ### Panics
    /// when called before JwtManager is set
    #[cfg(feature = "jwt")]
    pub fn jwt(&self) -> &JwtManager {
        self.jwt.as_ref().expect("JwtManager has not been set!")
    }

    /// Insert or replace a state value keyed by its concrete type.
    /// The value is stored as Arc<T>
    pub fn set_state<T: Send + Sync + 'static>(&self, value: T) {
        let mut map = self.inner.write();
        map.insert(TypeId::of::<T>(), Arc::new(value));
    }

    /// Try to fetch state by type. Returns `Some(Arc<T>)` if present.
    pub fn try_get_state<T: Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        let map = self.inner.read();
        let arc_any = map.get(&TypeId::of::<T>())?.clone();
        // Attempt to downcast the Arc<dyn Any + Send + Sync> into Arc<T>
        // This should succeed because we stored Arc<T> originally.
        Arc::downcast::<T>(arc_any).ok()
    }

    /// Get state by type, panics if missing.
    pub fn get_state<T: Send + Sync + 'static>(&self) -> Arc<T> {
        self.try_get_state::<T>().expect("state not found for requested type")
    }

    /// Remove a state value of type T. Returns true if removed.
    pub fn remove_state<T: Send + Sync + 'static>(&self) -> bool {
        let mut map = self.inner.write();
        map.remove(&TypeId::of::<T>()).is_some()
    }
}

impl Default for AppContext {
    fn default() -> Self {
        Self::new()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[derive(Debug, Clone, PartialEq)]
    struct Counter {
        count: i32,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct User {
        id: u64,
        name: String,
    }

    #[derive(Debug, Clone)]
    struct Config {
        port: u16,
        host: String,
    }

    #[test]
    fn test_set_and_get_state() {
        let ctx = AppContext::new();

        let counter = Counter {
            count: 42,
        };
        ctx.set_state(counter.clone());

        let retrieved = ctx.get_state::<Counter>();
        assert_eq!(*retrieved, counter);
    }

    #[test]
    fn test_try_get_state_some() {
        let ctx = AppContext::new();

        ctx.set_state(String::from("hello"));

        let result = ctx.try_get_state::<String>();
        assert!(result.is_some());
        assert_eq!(*result.unwrap(), "hello");
    }

    #[test]
    fn test_try_get_state_none() {
        let ctx = AppContext::new();

        let result = ctx.try_get_state::<String>();
        assert!(result.is_none());
    }

    #[test]
    #[should_panic(expected = "state not found for requested type")]
    fn test_get_state_panics_when_missing() {
        let ctx = AppContext::new();
        let _value = ctx.get_state::<String>();
    }

    #[test]
    fn test_multiple_different_types() {
        let ctx = AppContext::new();

        ctx.set_state(Counter {
            count: 10,
        });
        ctx.set_state(String::from("test"));
        ctx.set_state(vec![1, 2, 3, 4, 5]);
        ctx.set_state(42u64);

        assert_eq!(ctx.get_state::<Counter>().count, 10);
        assert_eq!(*ctx.get_state::<String>(), "test");
        assert_eq!(*ctx.get_state::<Vec<i32>>(), vec![1, 2, 3, 4, 5]);
        assert_eq!(*ctx.get_state::<u64>(), 42);
    }

    #[test]
    fn test_replace_state() {
        let ctx = AppContext::new();

        ctx.set_state(Counter {
            count: 5,
        });
        assert_eq!(ctx.get_state::<Counter>().count, 5);

        // Replace with new value
        ctx.set_state(Counter {
            count: 100,
        });
        assert_eq!(ctx.get_state::<Counter>().count, 100);
    }

    #[test]
    fn test_remove_state_exists() {
        let ctx = AppContext::new();

        ctx.set_state(String::from("will be removed"));
        assert!(ctx.try_get_state::<String>().is_some());

        let removed = ctx.remove_state::<String>();
        assert!(removed);
        assert!(ctx.try_get_state::<String>().is_none());
    }

    #[test]
    fn test_remove_state_not_exists() {
        let ctx = AppContext::new();

        let removed = ctx.remove_state::<String>();
        assert!(!removed);
    }

    #[test]
    fn test_arc_sharing() {
        let ctx = AppContext::new();

        ctx.set_state(Counter {
            count: 7,
        });

        let arc1 = ctx.get_state::<Counter>();
        let arc2 = ctx.get_state::<Counter>();

        // Both should point to the same data
        assert_eq!(arc1.count, arc2.count);
        assert_eq!(Arc::strong_count(&arc1), Arc::strong_count(&arc2));
    }

    #[test]
    fn test_clone_shares_state() {
        let ctx1 = AppContext::new();
        ctx1.set_state(User {
            id: 1,
            name: "Alice".to_string(),
        });

        let ctx2 = ctx1.clone();

        // Both contexts should access the same state
        let user1 = ctx1.get_state::<User>();
        let user2 = ctx2.get_state::<User>();

        assert_eq!(*user1, *user2);
    }

    #[test]
    fn test_state_isolation_by_type() {
        let ctx = AppContext::new();

        ctx.set_state(42i32);
        ctx.set_state(42u32);
        ctx.set_state(42i64);

        assert_eq!(*ctx.get_state::<i32>(), 42i32);
        assert_eq!(*ctx.get_state::<u32>(), 42u32);
        assert_eq!(*ctx.get_state::<i64>(), 42i64);

        ctx.remove_state::<i32>();

        assert!(ctx.try_get_state::<i32>().is_none());
        assert!(ctx.try_get_state::<u32>().is_some());
        assert!(ctx.try_get_state::<i64>().is_some());
    }

    #[test]
    fn test_complex_type() {
        let ctx = AppContext::new();

        let config = Config {
            port: 8080,
            host: "127.0.0.1".to_string(),
        };

        ctx.set_state(config.clone());

        let retrieved = ctx.get_state::<Config>();
        assert_eq!(retrieved.port, 8080);
        assert_eq!(retrieved.host, "127.0.0.1");
    }

    #[test]
    fn test_nested_arc_types() {
        let ctx = AppContext::new();

        let data = Arc::new(vec![1, 2, 3, 4]);
        ctx.set_state(data.clone());

        let retrieved = ctx.get_state::<Arc<Vec<i32>>>();
        assert_eq!(**retrieved, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_thread_safety() {
        use std::thread;

        let ctx = AppContext::new();
        ctx.set_state(AtomicUsize::new(0));

        let mut handles = vec![];

        for _ in 0..10 {
            let ctx_clone = ctx.clone();
            let handle = thread::spawn(move || {
                for _ in 0..100 {
                    let counter = ctx_clone.get_state::<AtomicUsize>();
                    counter.fetch_add(1, Ordering::SeqCst);
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let counter = ctx.get_state::<AtomicUsize>();
        assert_eq!(counter.load(Ordering::SeqCst), 1000);
    }

    #[test]
    fn test_concurrent_reads() {
        use std::thread;

        let ctx = AppContext::new();
        ctx.set_state(String::from("shared data"));

        let mut handles = vec![];

        for _ in 0..20 {
            let ctx_clone = ctx.clone();
            let handle = thread::spawn(move || {
                let data = ctx_clone.get_state::<String>();
                assert_eq!(*data, "shared data");
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_concurrent_set_and_get() {
        use std::sync::Barrier;
        use std::thread;

        let ctx = AppContext::new();
        let barrier = Arc::new(Barrier::new(5));

        let mut handles = vec![];

        for i in 0..5 {
            let ctx_clone = ctx.clone();
            let barrier_clone = barrier.clone();

            let handle = thread::spawn(move || {
                barrier_clone.wait();
                ctx_clone.set_state(format!("thread-{}", i));
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // One of the threads will win, verify we can read it
        let result = ctx.get_state::<String>();
        assert!(result.starts_with("thread-"));
    }

    #[test]
    fn test_default_constructor() {
        let ctx = AppContext::default();

        ctx.set_state(42i32);
        assert_eq!(*ctx.get_state::<i32>(), 42);
    }

    #[test]
    fn test_empty_context() {
        let ctx = AppContext::new();

        assert!(ctx.try_get_state::<String>().is_none());
        assert!(ctx.try_get_state::<i32>().is_none());
        assert!(ctx.try_get_state::<Vec<u8>>().is_none());
    }

    #[test]
    fn test_option_types() {
        let ctx = AppContext::new();

        ctx.set_state(Some(42i32));
        ctx.set_state(None::<String>);

        let some_value = ctx.get_state::<Option<i32>>();
        assert_eq!(*some_value, Some(42));

        let none_value = ctx.get_state::<Option<String>>();
        assert_eq!(*none_value, None);
    }

    #[test]
    fn test_result_types() {
        let ctx = AppContext::new();

        ctx.set_state(Ok::<i32, String>(42));
        ctx.set_state(Err::<i32, String>("error".to_string()));

        let ok_value = ctx.get_state::<Result<i32, String>>();
        assert_ne!(*ok_value, Ok(42));

        let err_value = ctx.get_state::<Result<i32, String>>();
        assert_eq!(*err_value, Err("error".to_string())); // Last set wins
    }

    #[test]
    fn test_tuple_types() {
        let ctx = AppContext::new();

        ctx.set_state((1, "hello".to_string(), true));

        let tuple = ctx.get_state::<(i32, String, bool)>();
        assert_eq!(tuple.0, 1);
        assert_eq!(tuple.1, "hello");
        assert_eq!(tuple.2, true);
    }

    #[test]
    fn test_large_number_of_types() {
        let ctx = AppContext::new();

        for i in 0..100 {
            ctx.set_state(format!("value-{}", i));
            // Each set replaces the previous String
        }

        let final_value = ctx.get_state::<String>();
        assert_eq!(*final_value, "value-99");
    }
}
