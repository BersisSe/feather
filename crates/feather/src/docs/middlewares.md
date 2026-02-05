# Middlewares in Feather
Middlewares are the core of Feather's request processing pipeline.
This guide covers everything you need to know about creating and using middlewares.

## What is a Middleware?

A middleware is a function that processes HTTP requests and responses. It sits in the request pipeline and can:

- Read request data (headers, body, path)
- Modify the response
- Control the flow (continue to next middleware or skip to next route)
- Access application state

## Middleware Trait

Feather defines middleware through the `Middleware` trait:

```rust,ignore
pub trait Middleware: Send + Sync {
    fn handle(&self, request: &mut Request, response: &mut Response, ctx: &AppContext) -> Outcome;
}
```
A middleware can:

- Inspect: Read headers, cookies, or bodies.
- Mutate: Add headers to the response or modify the request context.
- Intercept: Stop the request early (e.g., for Auth failures).
- Delegate: Pass the baton to the next handler in line.

But What is `Outcome`:
Outcome is a type definition around Result, allowing you to use the try operator (?) inside your logic for clean error propagation.
```rust,ignore
pub enum MiddlewareResult {
    Next,           // Continue to next middleware
    NextRoute,      // Skip to next route handler
    End,            // Stop Executing and Send the Request.
}
pub type Outcome = Result<MiddlewareResult, Box<dyn Error>>;
```


## Defining Middlewares

### The `#[middleware_fn]` Attribute (Recommended)

This is the cleanest way to write reusable logic. **As of Feather 0.8.0**, this macro is optimized to play perfectly with our new internal routing engine.

```rust,ignore
use feather::middleware_fn;

#[middleware_fn]
fn log_requests() {
    println!("[{}] {}", req.method, req.uri);
    next!() // Essential to move the chain forward!
}

app.use_middleware(log_requests);

```

### The `middleware!` Macro

Great for quick, inline logic where you don't want to define a standalone function.

```rust,ignore
app.get("/", middleware!(|_req, res, _ctx| {
    res.send_text("Quick response");
    next!()
}));

```
### Using Closures

You can use closures directly as they implement `Middleware` trait automaticly:

```rust,ignore
let my_middleware = |req: &mut Request, res: &mut Response, ctx: &AppContext| {
    println!("Processing request to: {}", req.uri);
    Ok(feather::MiddlewareResult::Next)
};

app.use_middleware(my_middleware);
```
But they are not really recommended anymore.

### Using Regular Functions

You can also implement the full middleware signature manually:

```rust,ignore
fn my_middleware(req: &mut Request, res: &mut Response, ctx: &AppContext) -> Outcome {
    println!("Processing request to: {}", req.uri);
    Ok(feather::MiddlewareResult::Next)
}

app.use_middleware(my_middleware);
```

### Struct-Based Middleware

For more complex logic, implement the `Middleware` trait on a struct:

```rust,ignore
use feather::{Middleware, MiddlewareResult, Outcome, Request, Response, AppContext};

struct LoggingMiddleware;

impl Middleware for LoggingMiddleware {
    fn handle(&self, req: &mut Request, res: &mut Response, _ctx: &AppContext) -> Outcome {
        println!("[{}] {}", req.method, req.uri);
        Ok(MiddlewareResult::Next)
    }
}

// Use it in your app
app.use_middleware(LoggingMiddleware);
```

## Scoped Middleware (New in 0.8.0)

One of the most powerful features as of Feather 0.8.0 is the ability to scope middleware to a specific `Router`. Unlike global middleware which runs on *every* request, scoped middleware only fires for routes mounted under that router.

```rust,ignore
let mut api_router = Router::new();

// This only runs for /api/* routes
api_router.use_middleware(|_req, _res, _ctx| {
    info!("Scoped API check...");
    next!()
});

api_router.get("/data", my_handler);
app.mount("/api", api_router);

```

## Control Flow: The "Next" Pattern

Control flow in Feather is explicit. You decide when the request continues or stops.

* **`next!()`**: The standard way to move to the next middleware in the current stack.
* **`next_route!()`**: **As of Feather 0.8.0**, this tells the engine to abandon the current route matching entirely and look for the next path that matches the request.
* **`end!()`**: Stops the chain immediately. Useful if you've already sent a response and don't want any further middleware (like loggers) to execute logic.


## Global Middleware

Apply middleware to all routes using `use_middleware()`:

```rust,ignore
// This runs on every request
app.use_middleware(middleware!(|req, res, _ctx| {
    println!("Request: {} {}", req.method, req.uri);
    next!()
}));

// This runs only on this route
app.get("/", middleware!(|_req, res, _ctx| {
    res.send_text("Hello");
    next!()
}));
```

Execution order:
1. Global middleware (in order defined)
2. Route-specific middleware(in order of registered)

## Practical Examples:

Here is a look at a real-world examoles using the modern v0.8.0 patterns:

### The Auth Guard

```rust,ignore
#[middleware_fn]
fn auth_guard() {
    let token = req.headers.get("Authorization");
    
    if let Some(t) = token {
        if t == "valid-token" {
            return next!(); // All good, keep going!
        }
    }

    // Stop them right here
    res.set_status(401).finish_text("Unauthorized")
}

```

### Logging Middleware

```rust,ignore
app.use_middleware(middleware!(|req, res, _ctx| {
    let method = req.method;
    let path = req.uri.clone();
    
    println!("→ {} {}", method, path);
    
    next!()
}));
```

### CORS Middleware

```rust,ignore
app.use_middleware(middleware!(|req, res, _ctx| {
    res.headers.append(
        "Access-Control-Allow-Origin",
        "*".parse().unwrap()
    );
    res.headers.append(
        "Access-Control-Allow-Methods",
        "GET, POST, PUT, DELETE, OPTIONS".parse().unwrap()
    );
    
    if req.method == feather_runtime::Method::OPTIONS {
        res.set_status(204);
        return Ok(MiddlewareResult::NextRoute);
    }
    
    next!()
}));
```

### Content-Type Validation

```rust,ignore
app.post("/api/data", middleware!(|req, res, _ctx| {
    if let Some(content_type) = req.headers.get("Content-Type") {
        if let Ok(ct) = content_type.to_str() {
            if ct.contains("application/json") {
                // Process JSON
                return next!();
            }
        }
    }
    
    res.set_status(400);
    res.send_text("Expected application/json");
    Ok(MiddlewareResult::NextRoute)
}));
```

### Request Body Processing

```rust,ignore
app.post("/echo", middleware!(|req, res, _ctx| {
    let body_str = String::from_utf8_lossy(&req.body);
    res.send_text(format!("Received: {}", body_str));
    next!()
}));
```

### Conditional Middleware

```rust,ignore
app.use_middleware(middleware!(|req, res, _ctx| {
    // Only apply to specific paths
    if req.uri.starts_with("/admin") {
        if !is_admin(req) {
            res.set_status(403);
            res.send_text("Forbidden");
            return next_route!()
        }
    }
    next!()
}));
```

## Accessing Application State

Use the `ctx` parameter to access application-wide state:

```rust,ignore
use feather::State;

#[derive(Clone)]
struct Config {
    api_key: String,
}

fn main() {
    let mut app = App::new();
    
    // Set state
    app.context().set_state(State::new(Config {
        api_key: "secret-key".to_string(),
    }));
    
    // Access state in middleware
    app.use_middleware(middleware!(|_req, res, ctx| {
        let config = ctx.get_state::<State<Config>>();
        let api_key = config.with_scope(|c| c.api_key.clone());
        
        res.send_text(format!("API Key: {}", api_key));
        next!()
    }));
}
```

See [State Management](./state-management.md) for more details.

## Error Handling in Middleware

You can return errors from middleware:

```rust,ignore
middleware!(|req, res, _ctx| {
    if req.body.is_empty() {
        res.set_status(400);
        res.send_text("Body required");
        return Ok(MiddlewareResult::NextRoute);
    }
    next!()
})
```

See [Error Handling](./error-handling.md) for comprehensive error handling.

## Chaining Middleware

Multiple middleware can be applied to a single route:

```rust,ignore
// While you can't chain directly, you can use global middleware
// combined with route-specific middleware

// Global middleware for all routes
app.use_middleware(middleware!(|req, res, _ctx| {
    // Validation 1
    next!()
}));

app.use_middleware(middleware!(|req, res, _ctx| {
    // Validation 2
    next!()
}));

// Route-specific middleware
app.post("/users", middleware!(|req, res, _ctx| {
    // Route handler
    next!()
}));
```

## JWT-Protected Middleware

Use the `#[jwt_required]` macro with `#[middleware_fn]` to automatically protect routes with JWT authentication:

```rust,ignore
use feather::{jwt_required, middleware_fn, Claim};

#[derive(Claim, Clone)]
struct UserClaims {
    #[required]
    user_id: String,
    username: String,
}

#[jwt_required]
#[middleware_fn]
fn get_profile(claims: UserClaims) {
    res.send_text(format!("Profile for: {}", claims.username));
    next!()
}

let mut app = App::new();
app.get("/profile", get_profile);
```

The `#[jwt_required]` macro automatically:
- Extracts the JWT token from the `Authorization: Bearer <token>` header
- Decodes and validates the token
- Validates claims using the `Claim` trait
- Returns 401 Unauthorized if anything fails

#### Example: Multi-Field Claims

```rust,ignore
#[derive(Claim, Clone)]
struct AuthClaims {
    #[required]
    user_id: String,
    #[required]
    email: String,
    role: String,
    #[exp]
    expires_at: usize,
}

#[jwt_required]
#[middleware_fn]
fn admin_only(claims: AuthClaims) {
    if claims.role != "admin" {
        res.set_status(403);
        res.send_text("Forbidden");
        return next!();
    }
    
    // Access context or state for additional checks
    res.send_text(format!("Admin panel for: {}", claims.email));
    next!()
}
```

See [Authentication](../authentication.md) for complete JWT setup and examples.

## Performance Tips

1. **Keep middleware lightweight** - Heavy processing should be done in route handlers
2. **Return early** - If a middleware can't proceed, return `next_route!` or `end!` immediately to save CPU cycles
3. **Cache state lookups** - If you access state multiple times, cache the result
4. **Use appropriate types** - Prefer references over clones when possible
