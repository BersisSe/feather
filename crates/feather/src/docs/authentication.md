# JWT Authentication in Feather

Feather includes built-in JWT (JSON Web Token) support for securing your APIs. This guide covers JWT authentication in Feather.

## Prerequisites

Enable the `jwt` feature in your `Cargo.toml`:

```toml
[dependencies]
feather = { version = "0.6", features = ["jwt"] }
```

## Setting Up JWT

### Creating a JWT Manager

Initialize JWT support in your application:

```rust
use feather::{App, jwt::JwtManager};

fn main() {
    let mut app = App::new();
    
    // Create a JWT manager with a secret key
    let secret = "your-super-secret-key-min-32-chars!".to_string();
    let jwt_manager = JwtManager::new(secret);
    
    // Set it in the app context
    app.context().set_jwt(jwt_manager);
    
    app.listen("127.0.0.1:5050");
}
```

## Token Generation

### Simple Tokens

Generate tokens with minimal setup:

```rust
use feather::jwt::JwtManager;

let jwt = JwtManager::new("secret-key".to_string());

// Generate a token with subject and TTL
match jwt.generate_simple("user123", 24) {  // 24 hour TTL
    Ok(token) => println!("Token: {}", token),
    Err(e) => println!("Error: {}", e),
}
```

### Custom Claims with Derive Macro

The easiest way to define custom claims is with the `#[derive(Claim)]` macro:

```rust
use feather::jwt::Claim;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Claim, Clone)]
pub struct UserClaims {
    #[required]
    pub sub: String,        // Subject (usually user ID)
    #[required]
    pub email: String,      // Custom field
    pub role: String,       // Custom field
    #[exp]
    pub exp: usize,         // Expiration time (automatically validated)
}
```

The `#[derive(Claim)]` macro automatically:
- Validates fields marked with `#[required]` are not empty
- Validates `#[exp]` fields are valid Unix timestamps in the future
- Implements the `Claim` trait for you

#### Manual Implementation (Advanced)

For custom validation logic, implement `Claim` manually:

```rust
use feather::jwt::Claim;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct AdminClaims {
    pub sub: String,
    pub exp: usize,
    pub admin_id: u64,
    pub permissions: Vec<String>,
}

impl Claim for AdminClaims {
    fn validate(&self) -> Result<(), jsonwebtoken::errors::Error> {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        if self.sub.is_empty() {
            return Err(jsonwebtoken::errors::Error::from(
                jsonwebtoken::errors::ErrorKind::InvalidToken
            ));
        }
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize;
        
        if self.exp < now {
            return Err(jsonwebtoken::errors::Error::from(
                jsonwebtoken::errors::ErrorKind::ExpiredSignature
            ));
        }
        
        Ok(())
    }
}
```

## Token Validation

### Decoding Tokens

Decode and validate tokens:

```rust
use feather::jwt::JwtManager;

let jwt = JwtManager::new("secret-key".to_string());
let token = "your-jwt-token";

match jwt.decode::<UserClaims>(token) {
    Ok(claims) => {
        println!("User ID: {}", claims.user_id);
        println!("Email: {}", claims.email);
    }
    Err(e) => {
        println!("Token validation failed: {}", e);
    }
}
```

## Protected Routes with JWT

### Using `#[jwt_required]` Macro (Recommended)

The easiest way to protect routes is with the `#[jwt_required]` and `#[middleware_fn]` macros:

```rust
use feather::{App, jwt::JwtManager, jwt_required, middleware_fn, Claim};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Claim, Clone)]
struct UserClaims {
    #[required]
    pub sub: String,
    pub email: String,
}

#[jwt_required]
#[middleware_fn]
fn get_profile(claims: UserClaims) {
    res.send_text(format!("Profile for: {}", claims.email));
    next!()
}

fn main() {
    let mut app = App::new();
    
    let jwt = JwtManager::new("secret-key".to_string());
    app.context().set_jwt(jwt);
    
    // Protected route - automatically validates JWT
    app.get("/api/profile", get_profile);
    
    app.listen("127.0.0.1:5050");
}
```

The `#[jwt_required]` macro automatically:
- Extracts the token from `Authorization: Bearer <token>` header
- Decodes and validates the token
- Validates claims (required fields, expiration)
- Returns 401 Unauthorized if anything fails

#### Multiple Protected Routes

```rust
#[jwt_required]
#[middleware_fn]
fn get_user_data(claims: UserClaims) {
    res.send_text(format!("Data for: {}", claims.sub));
    next!()
}

#[jwt_required]
#[middleware_fn]
fn update_user(claims: UserClaims) {
    res.send_text(format!("Updating user: {}", claims.sub));
    next!()
}

let mut app = App::new();
app.context().set_jwt(JwtManager::new("secret-key".to_string()));

app.get("/api/user", get_user_data);
app.put("/api/user", update_user);
```

### Manual JWT Protection (Advanced)

For custom error handling or conditional validation:

```rust
use feather::middleware;

app.get("/api/custom", middleware!(|req, res, ctx| {
    // Get token from Authorization header
    let token = match req.headers.get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer ")) {
        Some(t) => t,
        None => {
            res.set_status(401);
            res.send_text("Missing Authorization header");
            return next!();
        }
    };
    
    // Decode and validate token
    let jwt = ctx.jwt();
    let claims: UserClaims = match jwt.decode(token) {
        Ok(c) => c,
        Err(e) => {
            res.set_status(401);
            res.send_text(format!("Invalid token: {}", e));
            return next!();
        }
    };
    
    // Use claims in your handler
    res.send_text(format!("User: {}", claims.email));
    next!()
}));
```

### with_jwt_auth Middleware

Alternatively, use the `with_jwt_auth` middleware:

```rust
use feather::jwt::{with_jwt_auth, SimpleClaims};

app.get(
    "/protected",
    with_jwt_auth(|_req, res, _ctx, claims: SimpleClaims| {
        res.send_text(format!("Hello, {}!", claims.sub));
        next!()
    })
);
```

## Complete Authentication Flow

### Full Example with Login and Protected Routes

```rust
use feather::{App, jwt::JwtManager, jwt_required, middleware_fn, middleware, Claim};
use serde::{Deserialize, Serialize,json};

#[derive(Serialize, Deserialize, Claim, Clone)]
struct AuthClaims {
    #[required]
    sub: String,
    email: String,
    #[exp]
    exp: usize,
}

fn main() {
    let mut app = App::new();
    
    let jwt = JwtManager::new("your-secret-key".to_string());
    app.context().set_jwt(jwt);
    
    // Login endpoint (no auth required)
    app.post("/auth/login", middleware!(|req, res, ctx| {
        // Validate credentials (simplified)
        let body = String::from_utf8_lossy(&req.body);
        
        if body.contains("admin") && body.contains("password") {
            let jwt = ctx.jwt();
            
            // Generate token with 24 hour expiry
            match jwt.generate_simple("admin", 24) {
                Ok(token) => {
                    res.send_text(format!(r#"{{"token":"{}"}}"#, token));
                }
                Err(_) => {
                    res.set_status(500);
                    res.send_text("Failed to generate token");
                }
            }
        } else {
            res.set_status(401);
            res.send_text("Invalid credentials");
        }
        
        next!()
    }));
    
    // Protected endpoint using #[jwt_required]
    #[jwt_required]
    #[middleware_fn]
    fn get_profile(claims: AuthClaims) {
        res.send_text(format!(r#"{{"profile":"{}","email":"{}"}}"#, claims.sub, claims.email));
        next!()
    }
    
    app.get("/api/profile", get_profile);
    
    // Health check (no auth)
    app.get("/health", middleware!(|_req, res, _ctx| {
        res.send_json(json!({"status":"ok"}));
        next!()
    }));
    
    app.listen("127.0.0.1:5050");
}
```

### Alternative: Custom Validation Flow

```rust
#[derive(Serialize, Deserialize, Claim, Clone)]
struct AdminClaims {
    #[required]
    sub: String,
    role: String,
    #[exp]
    exp: usize,
}

#[jwt_required]
#[middleware_fn]
fn admin_only(claims: AdminClaims) {
    if claims.role != "admin" {
        res.set_status(403);
        res.send_text("Admin access required");
        return next!();
    }
    
    res.send_text(format!("Admin panel for: {}", claims.sub));
    next!()
}

let mut app = App::new();
app.get("/api/admin", admin_only);
```

## Token Claims

### Using `#[derive(Claim)]` (Recommended)

Define claims quickly with automatic validation:

```rust
use feather::jwt::Claim;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Claim, Clone)]
pub struct UserClaims {
    #[required]
    pub user_id: String,
    #[required]
    pub email: String,
    pub role: String,
    #[exp]
    pub exp: usize,
}

// Automatically validates:
// - user_id and email are not empty
// - exp is a valid Unix timestamp in the future
```

Validation attributes:
- `#[required]` - Field must not be empty
- `#[exp]` - Field must be a valid future Unix timestamp

### SimpleClaims

Built-in claims for quick use:

```rust
use feather::jwt::SimpleClaims;

pub struct SimpleClaims {
    pub sub: String,    // Subject (user ID)
    pub exp: usize,     // Expiration time (Unix timestamp)
}
```

### Custom Claims (Manual)

For advanced validation, implement `Claim` manually:

```rust
use feather::jwt::Claim;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct AdminClaims {
    pub sub: String,
    pub exp: usize,
    pub admin_id: u64,
    pub permissions: Vec<String>,
}

impl Claim for AdminClaims {
    fn validate(&self) -> Result<(), jsonwebtoken::errors::Error> {
        // Add custom validation logic
        Ok(())
    }
}
```

## Best Practices

### Security

1. **Use strong secrets** - Use cryptographically secure random strings
2. **Use environment variables** - Don't hardcode secrets
3. **Use HTTPS** - Always transmit tokens over secure connections
4. **Short expiration** - Use reasonable TTLs (15 minutes to 24 hours)
5. **Refresh tokens** - Consider implementing refresh token flow for longer sessions

```rust
use std::env;

fn main() {
    let secret = env::var("JWT_SECRET")
        .expect("JWT_SECRET environment variable not set");
    
    let jwt = JwtManager::new(secret);
    // ... rest of app
}
```

### Error Handling

Proper token error handling:

```rust
use feather::jwt::ErrorKind;

let token = "invalid-token";
let jwt = ctx.jwt();

match jwt.decode::<SimpleClaims>(token) {
    Ok(claims) => {
        // Token is valid
    }
    Err(e) => {
        if e.kind() == jsonwebtoken::errors::ErrorKind::ExpiredSignature {
            res.set_status(401);
            res.send_text("Token expired");
        } else if e.kind() == jsonwebtoken::errors::ErrorKind::InvalidToken {
            res.set_status(401);
            res.send_text("Invalid token");
        } else {
            res.set_status(401);
            res.send_text("Authentication failed");
        }
    }
}
```

### Token Expiration

Generate tokens with appropriate TTL:

```rust
// Short-lived access token (15 minutes)
jwt.generate_simple("user123", 1/96)?;  // 15 minutes = 1/96 of a day

// Regular session (24 hours)
jwt.generate_simple("user123", 1)?;

// Extended session (7 days)
jwt.generate_simple("user123", 7)?;

// Long-lived token (30 days)
jwt.generate_simple("user123", 30)?;
```

### Rate Limiting with JWT

Combine with rate limiting middleware:

```rust
use std::collections::HashMap;
use feather::State;

#[derive(Clone)]
struct RateLimiter {
    requests: HashMap<String, usize>,
}

fn main() {
    let mut app = App::new();
    
    let jwt = JwtManager::new("secret-key".to_string());
    app.context().set_jwt(jwt);
    
    app.context().set_state(State::new(RateLimiter {
        requests: HashMap::new(),
    }));
    
    app.use_middleware(middleware!(|req, res, ctx| {
        // Check rate limit for authenticated users
        if let Some(auth) = req.headers.get("Authorization") {
            if let Ok(auth_str) = auth.to_str() {
                let token = auth_str.strip_prefix("Bearer ").unwrap_or("");
                
                let jwt = ctx.jwt();
                if jwt.decode::<SimpleClaims>(token).is_ok() {
                    // Authenticated request
                    // Add rate limiting logic here
                }
            }
        }
        
        next!()
    }));
}
```

## Example Applications

### API Server with JWT (Using Macros)

```rust
use feather::{App, jwt::JwtManager, jwt_required, middleware_fn, middleware, Claim};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Claim, Clone)]
struct ApiUser {
    #[required]
    sub: String,
    email: String,
    #[exp]
    exp: usize,
}

fn main() {
    let mut app = App::new();
    
    let jwt = JwtManager::new("your-secret-key".to_string());
    app.context().set_jwt(jwt);
    
    // Health check (no auth)
    app.get("/health", middleware!(|_req, res, _ctx| {
        res.send_text(r#"{"status":"ok"}"#);
        next!()
    }));
    
    // Login endpoint
    app.post("/auth/login", middleware!(|_req, res, ctx| {
        let jwt = ctx.jwt();
        match jwt.generate_simple("user@example.com", 24) {
            Ok(token) => res.send_text(token),
            Err(_) => {
                res.set_status(500);
                res.send_text("Token generation failed");
            }
        }
        next!()
    }));
    
    // Protected endpoints using #[jwt_required]
    #[jwt_required]
    #[middleware_fn]
    fn get_user_data(user: ApiUser) {
        res.send_text(format!(r#"{{"user":"{}"}}"#, user.sub));
        next!()
    }
    
    #[jwt_required]
    #[middleware_fn]
    fn update_user(user: ApiUser) {
        res.send_text(format!("Updated user: {}", user.sub));
        next!()
    }
    
    app.get("/api/user", get_user_data);
    app.put("/api/user", update_user);
    
    app.listen("127.0.0.1:5050");
}
```

### API Server with JWT (Manual Approach)

```rust
use feather::{App, jwt::JwtManager, middleware, jwt::SimpleClaims};

fn main() {
    let mut app = App::new();
    
    let jwt = JwtManager::new("app-secret-key".to_string());
    app.context().set_jwt(jwt);
    
    // Health check (no auth)
    app.get("/health", middleware!(|_req, res, _ctx| {
        res.send_text(r#"{"status":"ok"}"#);
        next!()
    }));
    
    // Login (no auth)
    app.post("/auth/login", middleware!(|req, res, ctx| {
        if req.body.is_empty() {
            res.set_status(400);
            res.send_text("Credentials required");
            return next!();
        }
        
        let jwt = ctx.jwt();
        match jwt.generate_simple("user123", 24) {
            Ok(token) => res.send_text(token),
            Err(_) => {
                res.set_status(500);
                res.send_text("Token generation failed");
            }
        }
        
        next!()
    }));
    
    // Protected endpoint with manual validation
    app.get("/api/data", middleware!(|req, res, ctx| {
        let token = req.headers.get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .unwrap_or("");
        
        let jwt = ctx.jwt();
        match jwt.decode::<SimpleClaims>(token) {
            Ok(_) => res.send_json(json!({"data":"protected"})),
            Err(_) => {
                res.set_status(401);
                res.send_text("Unauthorized");
            }
        }
        
        next!()
    }));
    
    app.listen("127.0.0.1:5050");
}
```
