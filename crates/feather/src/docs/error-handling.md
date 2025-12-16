# Error Handling in Feather

Feather provides flexible error handling mechanisms. This guide covers all approaches to managing errors in your application.

## Default Error Handling

By default, Feather catches all errors and returns a 500 Internal Server Error response to the client:

```rust
use feather::App;

fn main() {
    let mut app = App::new();
    
    // If any error occurs, Feather returns 500
    app.get("/", middleware!(|_req, res, _ctx| {
        std::fs::File::open("non-existingfile.txt")?; // This will send a 500 response to the client
        next!()
    }));
    
    app.listen("127.0.0.1:5050");
}
```

The error is logged to `stderr` and a generic 500 response is sent to the client.

## Custom Error Handling

Set a custom error handler using `set_error_handler()`:

```rust
use feather::{App, ErrorHandler};

let error_handler: ErrorHandler = |err| {
    eprintln!("Application error: {}", err);
    // Return custom error response
    None  // Or Some(response) for custom responses
};

let mut app = App::new();
app.set_error_handler(error_handler);
```

## Handling Errors in Middleware

### Early Return on Error

Check conditions and return early:

```rust
app.post("/api/users", middleware!(|req, res, _ctx| {
    // Validate request
    if req.body.is_empty() {
        res.set_status(400);
        res.send_text("Request body is required");
        return next!();
    }
    
    // Process valid request
    res.send_text("User created");
    next!()
}));
```

### Status Code Responses

Send error status codes:

```rust
app.get("/resource/:id", middleware!(|req, res, _ctx| {
    // Simulate resource not found
    res.set_status(404);
    res.send_text("Resource not found");
    next!()
}));

app.post("/forbidden", middleware!(|_req, res, _ctx| {
    res.set_status(403);
    res.send_text("Access forbidden");
    next!()
}));

app.get("/error", middleware!(|_req, res, _ctx| {
    res.set_status(500);
    res.send_text("Internal server error");
    next!()
}));
```

### Conditional Error Handling

Handle different error scenarios:

```rust
app.post("/login", middleware!(|req, res, _ctx| {
    // Parse request body
    let body = String::from_utf8_lossy(&req.body);
    
    if body.is_empty() {
        res.set_status(400);
        res.send_text("Username and password required");
        return next!();
    }
    
    // Simulate authentication
    if body.contains("invalid") {
        res.set_status(401);
        res.send_text("Invalid credentials");
        return next!();
    }
    
    res.send_text("Login successful");
    next!()
}));
```

## Error Response Examples

### JSON Error Responses (with `json` feature)

```rust
#[cfg(feature = "json")]
app.post("/api/data", middleware!(|req, res, _ctx| {
    if req.body.is_empty() {
        res.set_status(400);
        res.send_json(feather::json!({
            "error": "Bad Request",
            "message": "Request body is required",
            "code": "EMPTY_BODY"
        }));
        return next!();
    }
    
    res.send_json(feather::json!({
        "status": "success",
        "data": null
    }));
    next!()
}));
```

### HTML Error Responses

```rust
app.get("/unknown", middleware!(|_req, res, _ctx| {
    res.set_status(404);
    res.send_html("<h1>404 Not Found</h1><p>The page you requested does not exist.</p>");
    next!()
}));
```

## Validation Patterns

### Request Validation

```rust
app.post("/users", middleware!(|req, res, _ctx| {
    // Check content type
    if let Some(ct) = req.headers.get("Content-Type") {
        if let Ok(ct_str) = ct.to_str() {
            if !ct_str.contains("application/json") {
                res.set_status(400);
                res.send_text("Content-Type must be application/json");
                return next!();
            }
        }
    }
    
    // Check body size
    if req.body.len() > 1024 * 1024 {  // 1MB limit
        res.set_status(413);
        res.send_text("Request body too large");
        return next!();
    }
    
    // If all validations pass
    res.send_text("User created");
    next!()
}));
```

### Schema Validation

```rust
app.post("/register", middleware!(|req, res, _ctx| {
    let body = String::from_utf8_lossy(&req.body);
    
    // Simple validation example
    if !body.contains("email") || !body.contains("password") {
        res.set_status(400);
        res.send_text("Missing required fields: email, password");
        return next!();
    }
    
    // Check email format (simplified)
    if !body.contains("@") {
        res.set_status(400);
        res.send_text("Invalid email format");
        return next!();
    }
    
    res.send_text("Registration successful");
    next!()
}));
```

## Error Middleware

Create middleware specifically for error handling:

```rust
// Global error handling middleware
app.use_middleware(middleware!(|req, res, ctx| {
    // Check for problematic requests
    if req.method.to_string() == "INVALID" {
        res.set_status(400);
        res.send_text("Invalid request method");
        return Ok(MiddlewareResult::NextRoute);
    }
    
    next!()
}));
```

## Common HTTP Status Codes

### Client Errors (4xx)

- **400** - Bad Request (malformed request)
- **401** - Unauthorized (authentication required)
- **403** - Forbidden (authenticated but not allowed)
- **404** - Not Found (resource doesn't exist)
- **405** - Method Not Allowed (wrong HTTP method)
- **409** - Conflict (request conflicts with current state)
- **413** - Payload Too Large (request body too big)
- **415** - Unsupported Media Type (wrong content type)
- **422** - Unprocessable Entity (validation failed)
- **429** - Too Many Requests (rate limited)

### Server Errors (5xx)

- **500** - Internal Server Error (server-side error)
- **502** - Bad Gateway (upstream error)
- **503** - Service Unavailable (server temporarily unavailable)
- **504** - Gateway Timeout (upstream timeout)

## Error Recovery Patterns

### Graceful Degradation

```rust
app.get("/data", middleware!(|_req, res, _ctx| {
    // Try to fetch data, fall back to default
    let data = match fetch_data() {
        Ok(d) => d,
        Err(_) => {
            res.set_status(503);  // Service Unavailable
            res.send_text("Service temporarily unavailable, using cached data");
            return next!();
        }
    };
    
    res.send_text(format!("Data: {}", data));
    next!()
}));

fn fetch_data() -> Result<String, Box<dyn std::error::Error>> {
    // Simulate data fetching
    Ok("data".to_string())
}
```

### Retry Logic

```rust
fn call_external_service(max_retries: usize) -> Result<String, String> {
    for attempt in 1..=max_retries {
        match try_service_call() {
            Ok(result) => return Ok(result),
            Err(e) if attempt < max_retries => {
                eprintln!("Attempt {} failed: {}, retrying...", attempt, e);
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            Err(e) => return Err(e),
        }
    }
    unreachable!()
}

fn try_service_call() -> Result<String, String> {
    Err("Connection timeout".to_string())
}

app.get("/external", middleware!(|_req, res, _ctx| {
    match call_external_service(3) {
        Ok(data) => res.send_text(data),
        Err(e) => {
            res.set_status(503);
            res.send_text(format!("Service unavailable: {}", e));
        }
    }
    next!()
}));
```

## Logging Errors

Use the logging feature to log errors:

```rust
#[cfg(feature = "log")]
app.use_middleware(middleware!(|_req, res, _ctx| {
    // Errors can be logged
    feather::warn!("Potential issue detected");
    next!()
}));
```

## Error Context

Include helpful context in error responses:

```rust
app.post("/api/update", middleware!(|req, res, _ctx| {
    let path = req.uri.clone();
    
    if !validate_request(req) {
        res.set_status(400);
        res.send_text(format!(
            "Invalid request to {}: Missing required fields",
            path
        ));
        return next!();
    }
    
    res.send_text("Updated successfully");
    next!()
}));

fn validate_request(req: &feather_runtime::http::Request) -> bool {
    !req.body.is_empty()
}
```

## Best Practices

1. **Always return a status code** - Help clients understand what happened
2. **Provide clear error messages** - Users should know what went wrong
3. **Log errors appropriately** - But don't expose internal details to clients
4. **Validate early** - Fail fast on invalid requests
5. **Use appropriate status codes** - Choose codes that match your error situation
6. **Handle all paths** - Ensure every code path has error handling
7. **Test error cases** - Don't just test the happy path
