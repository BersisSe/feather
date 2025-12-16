# Routing in Feather

Feather provides a simple, Express.js-like routing system. This guide covers everything you need to know about routing.

## Basic Routing

Define routes using HTTP method functions on the `App` struct:

```rust
use feather::App;

let mut app = App::new();

// GET request
app.get("/", middleware!(|_req, res, _ctx| {
    res.send_text("GET /");
    next!()
}));

// POST request
app.post("/users", middleware!(|_req, res, _ctx| {
    res.send_text("POST /users");
    next!()
}));

// PUT request
app.put("/users/:id", middleware!(|_req, res, _ctx| {
    res.send_text("PUT /users/:id");
    next!()
}));

// DELETE request
app.delete("/users/:id", middleware!(|_req, res, _ctx| {
    res.send_text("DELETE /users/:id");
    next!()
}));

// PATCH request
app.patch("/items/:id", middleware!(|_req, res, _ctx| {
    res.send_text("PATCH /items/:id");
    next!()
}));

// HEAD request
app.head("/status", middleware!(|_req, res, _ctx| {
    res.set_status(200);
    next!()
}));

// OPTIONS request
app.options("/api/*", middleware!(|_req, res, _ctx| {
    res.send_text("OPTIONS allowed");
    next!()
}));
```

## Supported HTTP Methods

Feather supports all standard HTTP methods:

- **GET** - `app.get(path, middleware)`
- **POST** - `app.post(path, middleware)`
- **PUT** - `app.put(path, middleware)`
- **DELETE** - `app.delete(path, middleware)`
- **PATCH** - `app.patch(path, middleware)`
- **HEAD** - `app.head(path, middleware)`
- **OPTIONS** - `app.options(path, middleware)`

## Path Parameters

Extract parameters from the URL using the `:paramName` syntax:

```rust
app.get("/users/:id", middleware!(|req, res, _ctx| {
    // The framework captures the path structure
    let user = req.param("id"); // Returns a option
    
    res.send_text(format!("User ID from path {}", user.unwrap())); 
    next!()
}));

app.get("/posts/:postId/comments/:commentId", middleware!(|_req, res, _ctx| {
    let post_id = req.param("postId")
    let comment_id = req.param("commentId")
    res.send_text("Post and comment IDs");
    next!()
}));
```

**Note**: While the path pattern includes parameters, Feather's current routing matches based on the path structure. For production use with complex parameter extraction, consider parsing the `req.uri` directly.

## Generic Route Definition

For advanced use cases, use the generic `route()` method:

```rust
use feather::Method;

app.route(Method::GET, "/custom", middleware!(|_req, res, _ctx| {
    res.send_text("Custom route");
    next!()
}));

app.route(Method::POST, "/api/data", middleware!(|_req, res, _ctx| {
    res.send_text("API data handler");
    next!()
}));
```
**Note**: Multi Method routing is in the works!

## Wildcard Routes

Use wildcards (`*`) to match any path structure:

```rust
// Match any path starting with /api/
app.get("/api/*", middleware!(|_req, res, _ctx| {
    res.send_text("API route");
    next!()
}));

// Catch-all route
app.get("/*", middleware!(|_req, res, _ctx| {
    res.set_status(404);
    res.send_text("Not found");
    next!()
}));
```

## Accessing Request Information

Inside your middleware, use `req` to access request data:

```rust
app.post("/api/data", middleware!(|req, res, _ctx| {
    // HTTP method
    println!("Method: {:?}", req.method);
    
    // Request URI/path
    println!("Path: {}", req.uri);
    
    // Headers
    if let Some(content_type) = req.headers.get("Content-Type") {
        println!("Content-Type: {:?}", content_type);
    }
    
    // Request body
    let body_bytes = &req.body;
    println!("Body length: {}", body_bytes.len());
    
    next!()
}));
```

## Route Groups

Create organized route groups using global middleware:

```rust
// API routes with common prefix handling
app.use_middleware(middleware!(|req, res, _ctx| {
    // Add CORS headers for API routes
    if req.uri.starts_with("/api") {
        res.headers.append("Access-Control-Allow-Origin", "*".parse().unwrap());
    }
    next!()
}));

app.get("/api/users", middleware!(|_req, res, _ctx| {
    res.send_text("Users list");
    next!()
}));

app.get("/api/posts", middleware!(|_req, res, _ctx| {
    res.send_text("Posts list");
    next!()
}));
```

## Status Codes and Responses

Set custom HTTP status codes:

```rust
app.post("/users", middleware!(|_req, res, _ctx| {
    res.set_status(201)  // Created
       .send_text("User created successfully");
    next!()
}));

app.get("/forbidden", middleware!(|_req, res, _ctx| {
    res.set_status(403);  // Forbidden
       .send_text("Access denied");
    next!()
}));

app.get("/not-found", middleware!(|_req, res, _ctx| {
    res.set_status(404);  // Not Found
       .send_text("Resource not found");
    next!()
}));
```

Common HTTP status codes:
- **200** - OK
- **201** - Created
- **204** - No Content
- **301** - Moved Permanently
- **302** - Found (Redirect)
- **304** - Not Modified
- **400** - Bad Request
- **401** - Unauthorized
- **403** - Forbidden
- **404** - Not Found
- **500** - Internal Server Error

## Example: RESTful API Routes

```rust
use feather::App;

fn main() {
    let mut app = App::new();
    
    // List all items
    app.get("/items", middleware!(|_req, res, _ctx| {
        res.send_text("[{id: 1, name: 'Item 1'}]");
        next!()
    }));
    
    // Get single item
    app.get("/items/:id", middleware!(|_req, res, _ctx| {
        res.send_text("{id: 1, name: 'Item 1'}");
        next!()
    }));
    
    // Create item
    app.post("/items", middleware!(|_req, res, _ctx| {
        res.set_status(201);
        res.send_text("{id: 2, name: 'Item 2'}");
        next!()
    }));
    
    // Update item
    app.put("/items/:id", middleware!(|_req, res, _ctx| {
        res.send_text("{id: 1, name: 'Updated Item'}");
        next!()
    }));
    
    // Delete item
    app.delete("/items/:id", middleware!(|_req, res, _ctx| {
        res.set_status(204);
        next!()
    }));
    
    app.listen("127.0.0.1:5050");
}
```
