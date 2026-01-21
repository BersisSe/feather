# State Management in Feather

Feather provides a powerful context-based state management system. Learn how to store and access application-wide state.

## What is AppContext?

`AppContext` is a type-safe, thread-safe container for application state. Every request has access to the same context, allowing you to:

- Store configuration
- Maintain databases connections
- Share counters and metrics
- Manage user sessions
- Store any application-wide data

## Creating State

### Basic State Setup

```rust,ignore
use feather::{App, AppContext};

#[derive(Clone)]
struct AppConfig {
    database_url: String,
    api_key: String,
}

fn main() {
    let mut app = App::new();
    
    // Create and store state
    let config = AppConfig {
        database_url: "postgresql://localhost/db".to_string(),
        api_key: "secret-key".to_string(),
    };
    
    app.context().set_state(config); // inner data is provided by a Arc Pointer for Read-Only data you dont need to use `State`
}
```

### State Wrapper

For mutable state, wrap it in the `State<T>` struct:

```rust,ignore
use feather::State;

#[derive(Clone)]
struct Counter {
    count: i32,
}

app.context().set_state(State::new(Counter { count: 0 }));
```

For read-only state, you can store it directly:

```rust,ignore
#[derive(Clone)]
struct Config {
    name: String,
}

app.context().set_state(Config { 
    name: "MyApp".to_string() 
});
```

## Accessing State

### From Middleware

Access state using the `ctx` parameter:

```rust,ignore
app.get("/", middleware!(|_req, res, ctx| {
    let config = ctx.get_state::<State<AppConfig>>();
    config.with_scope(|c|{
        // State is now available in this scope
    });
    next!()
}));
```

### Type-Safe Retrieval

Feather uses the type system to manage state. Each state value is keyed by its type:

```rust,ignore
// Different types are stored separately
app.context().set_state(State::new(Config { ... }));
app.context().set_state(State::new(Counter { ... }));

// Access by type
let config = ctx.get_state::<State<Config>>();
let counter = ctx.get_state::<State<Counter>>();
```

### Cloning State

If your state type implements `Clone`, you can get a clone:

```rust,ignore
#[derive(Clone)]
struct User {
    id: u64,
    name: String,
}

let user_clone = ctx.get_state::<State<User>>().get_clone();

```

## Working with Mutable State

### with_mut_scope

The most ergonomic way to modify state:

```rust,ignore
#[derive(Clone)]
struct Counter {
    count: i32,
}

impl Counter {
    fn increment(&mut self) {
        self.count += 1;
    }
}

// In your middleware
let counter = ctx.get_state::<State<Counter>>();
counter.with_mut_scope(|c| {
    c.increment();
    c.increment();
});
```

### with_scope

For read-only accesses its cheaper and safer to use then `with_mut_scope`:
```rust,ignore
let counter = ctx.get_state::<State<Counter>>();
let current = counter.with_scope(|c| {
    println!("Current count: {}", c.count);
    c.count
});
```

### lock()

Get direct access to the lock guard.:

```rust,ignore
let counter = ctx.get_state::<State<Counter>>();
{
    let mut guard = counter.lock();
    guard.count += 1;
    guard.count += 1;  // Multiple operations with one lock
}
```

## Safe Concurrent Access

The `State<T>` wrapper uses `parking_lot::Mutex` for thread-safe access:

```rust,ignore
use feather::State;


#[derive(Clone)]
struct SharedData {
    value: String,
}

// State is automatically thread-safe
app.context().set_state(State::new(SharedData {
    value: "shared".to_string(),
}));

// Multiple threads can safely access it
app.get("/path1", middleware!(|_req, res, ctx| {
    let data = ctx.get_state::<State<SharedData>>();
    data.with_scope(|d| {
        println!("{}", d.value);
    });
    next!()
}));

app.get("/path2", middleware!(|_req, res, ctx| {
    let data = ctx.get_state::<State<SharedData>>();
    data.with_scope(|d| {
        println!("{}", d.value);
    });
    next!()
}));
```

## Optional State Access

Use `try_get_state()` to handle missing state:

```rust,ignore
let maybe_config = ctx.try_get_state::<State<Config>>();

match maybe_config {
    Some(config) => {
        config.with_scope(|c| {
            println!("Config found: {:?}", c);
        });
    }
    None => {
        res.send_text("Configuration not available");
    }
}
```

## Removing State

Remove state from context:

```rust,ignore
// Removes State<Config> if it exists
let removed = ctx.remove_state::<State<Config>>();

if removed {
    println!("State was removed");
} else {
    println!("State was not found");
}
```

## Common State Patterns

### Database Connection Pool

```rust,ignore
use feather::State;

#[derive(Clone)]
struct Database {
    connection_string: String,
}

impl Database {
    fn query(&self, sql: &str) -> Vec<String> {
        // Execute query
        vec![]
    }
}

fn main() {
    let mut app = App::new();
    
    let db = Database {
        connection_string: "postgresql://localhost/db".to_string(),
    };
    
    app.context().set_state(State::new(db));
    
    app.get("/users", middleware!(|_req, res, ctx| {
        let db = ctx.get_state::<State<Database>>();
        let users = db.with_scope(|database| {
            database.query("SELECT * FROM users")
        });
        
        res.send_text(format!("Users: {:?}", users));
        next!()
    }));
}
```

### Configuration Management

```rust,ignore
#[derive(Clone)]
struct Config {
    port: u16,
    host: String,
    debug: bool,
}

fn main() {
    let mut app = App::new();
    
    app.context().set_state(State::new(Config {
        port: 5050,
        host: "127.0.0.1".to_string(),
        debug: true,
    }));
}
```

### Metrics and Counters

```rust,ignore
#[derive(Clone)]
struct Metrics {
    requests: i64,
    errors: i64,
}

impl Metrics {
    fn record_request(&mut self) {
        self.requests += 1;
    }
    
    fn record_error(&mut self) {
        self.errors += 1;
    }
}

fn main() {
    let mut app = App::new();
    
    app.context().set_state(State::new(Metrics {
        requests: 0,
        errors: 0,
    }));
    
    // Record metrics in middleware
    app.use_middleware(middleware!(|_req, res, ctx| {
        let metrics = ctx.get_state::<State<Metrics>>();
        metrics.with_mut_scope(|m| {
            m.record_request();
        });
        next!()
    }));
}
```

### User Sessions (Simple Example)

```rust,ignore
use std::collections::HashMap;
use feather::State;

#[derive(Clone)]
struct Session {
    user_id: Option<u64>,
}

#[derive(Clone)]
struct Sessions {
    sessions: HashMap<String, Session>,
}

fn main() {
    let mut app = App::new();
    
    app.context().set_state(State::new(Sessions {
        sessions: HashMap::new(),
    }));
}
```

## State Lifetimes

State in Feather lives for the entire duration of the application:

```rust,ignore
let mut app = App::new();

// State stored here...
app.context().set_state(State::new(MyData { ... }));

// ...is available for all requests until app.listen() is called
app.listen("127.0.0.1:5050");  // Blocks forever
```

When `app.listen()` is called, the app starts serving requests and continues until the process exits.

## Deadlock Prevention

⚠️ **Important**: Do NOT access the same `State<T>` recursively:

```rust,ignore
// DON'T DO THIS - Will cause deadlock!
let state = ctx.get_state::<State<MyType>>();
state.with_scope(|data| {
    let state_again = ctx.get_state::<State<MyType>>();  // ❌ DEADLOCK!
    state_again.with_scope(|_| { /* ... */ });
});
```

Instead, refactor to avoid nested access:

```rust,ignore
// DO THIS - No deadlock
let state = ctx.get_state::<State<MyType>>();
let value = state.with_scope(|data| {
    // Extract what you need
    data.some_field.clone()
});

// Now you can access again if needed
let state = ctx.get_state::<State<MyType>>();
state.with_scope(|data| { /* ... */ });
```

## Best Practices

1. **Use `Clone` for state types if you can** - Makes it easier to work with
2. **Keep state simple** - Avoid deeply nested structures
3. **Use `with_scope()` or `with_mut_scope()`** - More ergonomic than `lock()`
4. **Extract values when needed** - Don't hold locks longer than necessary
5. **Use appropriate access patterns** - Read-only vs. mutable access
6. **Avoid recursive access** - Can cause deadlocks
