# Getting Started with Feather

This guide will help you set up a basic Feather application and understand the core concepts.

## Installation

Add Feather to your `Cargo.toml`:

```toml
[dependencies]
feather = "0.6"
```

## Creating Your First App

The simplest Feather application looks like this:

```rust,ignore
use feather::App;

fn main() {
    let mut app = App::new();
    // Routes will be added here
    app.listen("127.0.0.1:5050");
}
```

## Adding Routes

Feather supports all standard HTTP methods through convenient methods:

```rust,ignore
use feather::{App, middleware, next};

fn main() {
    let mut app = App::new();
    
    // GET request
    app.get("/", middleware!(|_req, res, _ctx| {
        res.send_text("Hello, World!");
        next!()
    }));
    
    // POST request
    app.post("/users", middleware!(|req, res, _ctx| {
        // Handle POST
        res.set_status(201);
        next!()
    }));
    
    // Other methods
    app.put("/users/:id", middleware!(|_req, res, _ctx| {
        res.send_text("PUT request");
        next!()
    }));
    
    app.delete("/users/:id", middleware!(|_req, res, _ctx| {
        res.send_text("DELETE request");
        next!()
    }));
    
    app.patch("/items/:id", middleware!(|_req, res, _ctx| {
        res.send_text("PATCH request");
        next!()
    }));
    
    app.listen("127.0.0.1:5050");
}
```

## Understanding the Middleware Pattern

Every route handler is a middleware. The `middleware!` macro is a convenient way to define closures:

```rust,ignore
middleware!(|req, res, ctx| {
    // Process the request
    // Modify the response
    next!() // Continue to next middleware or finish
})
```

### Parameters

1. **`req: &mut Request`** - The incoming HTTP request with headers, body, and metadata
2. **`res: &mut Response`** - The HTTP response object to send back to the client
3. **`ctx: &AppContext`** - Application context for accessing shared state

### Return Value

The middleware returns an `Outcome` (which is a `Result`):
- `Ok(MiddlewareResult::Next)` - Continue to next middleware (via `next!()`)
- `Ok(MiddlewareResult::NextRoute)` - Skip to next route

## Responding to Requests

### Sending Text

```rust,ignore
app.get("/", middleware!(|_req, res, _ctx| {
    res.send_text("Hello, World!");
    next!()
}));
```

### Setting Status Codes

```rust,ignore
app.post("/users", middleware!(|_req, res, _ctx| {
    res.set_status(201);
    res.send_text("User created");
    next!()
}));
```

### Sending JSON (with `json` feature)

```rust,ignore
#[cfg(feature = "json")]
app.get("/api/data", middleware!(|_req, res, _ctx| {
    res.send_json(feather::json!({
        "status": "ok",
        "data": [1, 2, 3]
    }));
    next!()
}));
```

## Working with Request Data

```rust,ignore
app.post("/data", middleware!(|req, res, _ctx| {
    // Get headers
    if let Some(content_type) = req.headers.get("Content-Type") {
        res.send_text(format!("Content-Type: {:?}", content_type));
    }
    
    // Get request path and method
    println!("Method: {:?}, Path: {:?}", req.method, req.uri);
    
    // Get request body (as bytes)
    let body = &req.body;
    
    next!()
}));
```

## Application Context

Every Feather application has a context for managing global state. Access it with:

```rust,ignore
let ctx = app.context();
```

See [State Management](../state_management/index.html) for detailed information.

## Middleware vs Routes

- **Routes**: HTTP method + path specific handlers (GET /users, POST /data, etc.)
- **Global Middleware**: Applied to all routes before route-specific handlers

```rust,ignore
// Global middleware - runs on every request
app.use_middleware(middleware!(|req, res, _ctx| {
    println!("Request to: {}", req.uri);
    next!()
}));

// Route-specific middleware
app.get("/", middleware!(|_req, res, _ctx| {
    res.send_text("Home page");
    next!()
}));
```

## Error Handling

By default, Feather catches errors and returns a 500 status. You can customize this:

See [Error Handling](../error_handling/index.html) for detailed information.

## Server Configuration

Customize server behavior:

```rust,ignore
use feather::{App, ServerConfig};

let config = ServerConfig {
    max_body_size: 10 * 1024 * 1024,  // 10MB
    read_timeout_secs: 60,             // 60 seconds
    workers: 4,                        // 4 worker threads
    stack_size: 128 * 1024,            // 128KB
};

let mut app = App::with_config(config);
```

Or use convenience methods:

```rust,ignore
app.max_body(10 * 1024 * 1024);
app.read_timeout(60);
app.workers(4);
app.stack_size(128 * 1024);
```

See [Server Configuration](../server_configuration/index.html) for more details.

