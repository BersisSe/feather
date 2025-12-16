# Middlewares in Feather

Middlewares are the core of Feather's request processing pipeline. This guide covers everything you need to know about creating and using middlewares.

## What is a Middleware?

A middleware is a function that processes HTTP requests and responses. It sits in the request pipeline and can:

- Read request data (headers, body, path)
- Modify the response
- Control the flow (continue to next middleware or skip to next route)
- Access application state

## Middleware Trait

Feather defines middleware through the `Middleware` trait:

```rust
pub trait Middleware: Send + Sync {
    fn handle(&self, request: &mut Request, response: &mut Response, ctx: &AppContext) -> Outcome;
}
```

Where `Outcome` is:
```rust
pub enum MiddlewareResult {
    Next,           // Continue to next middleware
    NextRoute,      // Skip to next route handler
}
pub type Outcome = Result<MiddlewareResult, Box<dyn Error>>;
```

## Defining Middlewares

### Using the `middleware!` Macro

The simplest way to define middleware is with the `middleware!` macro:

```rust
use feather::{middleware, next};

app.get("/", middleware!(|req, res, ctx| {
    // Your middleware logic here
    next!()  // Continue to next middleware
}));
```

### Using Closures

You can use closures directly (they implement `Middleware`):

```rust
let my_middleware = |req: &mut Request, res: &mut Response, ctx: &AppContext| {
    println!("Processing request to: {}", req.uri);
    Ok(feather::MiddlewareResult::Next)
};

app.use_middleware(my_middleware);
```

### Using the `#[middleware_fn]` Attribute Macro

For reusable, named middleware functions, use the `#[middleware_fn]` attribute macro. This eliminates boilerplate by automatically injecting `req`, `res`, and `ctx` parameters:

```rust
use feather::middleware_fn;

#[middleware_fn]
fn log_requests() {
    println!("[{}] {}", req.method, req.uri);
    next!()
}

app.use_middleware(log_requests);
```

The macro automatically provides:
- `req: &mut Request` - The HTTP request
- `res: &mut Response` - The HTTP response
- `ctx: &AppContext` - Application context for accessing state

#### Why Use `#[middleware_fn]`?

- **Less boilerplate** - No need to repeat type signatures
- **More readable** - Focus on logic, not types
- **Reusable** - Define once, use on multiple routes
- **Clear intent** - Middleware functions are explicitly named

#### Comparison: `middleware!` vs `#[middleware_fn]`

```rust
// Use middleware! for inline, one-off middleware
app.get("/", middleware!(|_req, res, _ctx| {
    res.send_text("Hello");
    next!()
}));

// Use #[middleware_fn] for reusable middleware
#[middleware_fn]
fn greet() {
    res.send_text("Hello");
    next!()
}

app.use_middleware(greet);
app.get("/hello", greet);
app.post("/hi", greet);
```

#### Full Example with State Access

```rust
use feather::{middleware_fn, State};

#[derive(Clone)]
struct AppConfig {
    version: String,
}

#[middleware_fn]
fn add_version_header() {
    let config = ctx.get_state::<State<AppConfig>>();
    let version = config.with_scope(|c| c.version.clone());
    res.headers.insert("X-App-Version", version.parse().unwrap());
    next!()
}

fn main() {
    let mut app = App::new();
    app.context().set_state(State::new(AppConfig {
        version: "1.0.0".to_string(),
    }));
    
    app.use_middleware(add_version_header);
}
```

### Using Regular Functions

You can also implement the full middleware signature manually:

```rust
fn my_middleware(req: &mut Request, res: &mut Response, ctx: &AppContext) -> Outcome {
    println!("Processing request to: {}", req.uri);
    Ok(feather::MiddlewareResult::Next)
}

app.use_middleware(my_middleware);
```

### Struct-Based Middleware

For more complex logic, implement the `Middleware` trait on a struct:

```rust
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

## Control Flow

### Continue to Next Middleware

Use `next!()` to pass control to the next middleware:

```rust
middleware!(|req, res, ctx| {
    println!("Before next middleware");
    next!()  // Continue
})
```

### Skip to Next Route

Use `MiddlewareResult::NextRoute` to skip remaining middleware and go to the next route:

```rust
middleware!(|req, res, ctx| {
    if !is_authenticated(req) {
        res.set_status(401);
        res.send_text("Unauthorized");
        return Ok(MiddlewareResult::NextRoute);
    }
    next!()
})
```

## Global Middleware

Apply middleware to all routes using `use_middleware()`:

```rust
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

## Practical Examples

### Authentication Middleware

```rust
fn is_valid_token(token: &str) -> bool {
    token == "secret-token"
}

app.use_middleware(middleware!(|req, res, _ctx| {
    if req.uri.starts_with("/api/admin") {
        if let Some(auth) = req.headers.get("Authorization") {
            if let Ok(auth_str) = auth.to_str() {
                if is_valid_token(auth_str) {
                    return next!();
                }
            }
        }
        res.set_status(401);
        res.send_text("Unauthorized");
        return Ok(MiddlewareResult::NextRoute);
    }
    next!()
}));
```

### Logging Middleware

```rust
app.use_middleware(middleware!(|req, res, _ctx| {
    let method = req.method;
    let path = req.uri.clone();
    
    println!("→ {} {}", method, path);
    
    next!()
}));
```

### CORS Middleware

```rust
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

```rust
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

```rust
app.post("/echo", middleware!(|req, res, _ctx| {
    let body_str = String::from_utf8_lossy(&req.body);
    res.send_text(format!("Received: {}", body_str));
    next!()
}));
```

### Conditional Middleware

```rust
app.use_middleware(middleware!(|req, res, _ctx| {
    // Only apply to specific paths
    if req.uri.starts_with("/admin") {
        if !is_admin(req) {
            res.set_status(403);
            res.send_text("Forbidden");
            return Ok(MiddlewareResult::NextRoute);
        }
    }
    next!()
}));
```

## Accessing Application State

Use the `ctx` parameter to access application-wide state:

```rust
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

```rust
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

```rust
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

```rust
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

```rust
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
2. **Return early** - If a middleware can't proceed, return `NextRoute` immediately
3. **Cache state lookups** - If you access state multiple times, cache the result
4. **Use appropriate types** - Prefer references over clones when possible
