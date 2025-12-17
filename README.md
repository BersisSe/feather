<h1 align="center">🪶 Feather</h1>

<p align="center">
  <a href="https://crates.io/crates/feather"><img src="https://img.shields.io/crates/v/feather.svg" alt="Crates.io"/></a>
  <a href="https://docs.rs/feather"><img src="https://docs.rs/feather/badge.svg" alt="Docs.rs"/></a>
  <a href="./LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"/></a>
</p>

## **Feather** is a lightweight, DX-first web framework for Rust. Inspired by the simplicity of Express.js, but designed for Rust’s performance and safety.

## Why Feather?

- **Middleware-First Architecture**  
  Everything is a middleware—even if it's not, it produces a middleware in the end.  
  The new `middleware!` macro makes writing route and middleware closures concise and ergonomic.

- **Easy State Management Using Context**  
  The Context API makes it very easy to manage state without the use of Extractors/Macros.  

- **All in One**  
  Feather is a complete web framework that includes routing, middleware, logging, JWT authentication, and more, all in one package.

- **Feel of Async Without Async**  
  Feather is multithreaded by default, running on **Feather-Runtime**.
  
- **Great Documantation**  
  Feather is Fully Documented in [Docs.rs](https://docs.rs/feather/latest/feather/), I bet you could learn all of Feather in just a few hours or so.


## How it works behind the scenes

Feather is powered by **Feather-Runtime**, a custom runtime built for high concurrency and low latency without using Rust's async/await. Each request is handled by a lightweight coroutine(using `may`), enabling thousands of concurrent connections with simple, synchronous code. For more technical details, see [Feather-Runtime](./crates/feather-runtime).

---

## Getting Started

Add Feather to your `Cargo.toml`:

```toml
[dependencies]
feather = "~0.7"
```

---

## Quick Example


```rust
use feather::{App, middleware_fn, next};

#[middleware_fn]
fn hello() -> feather::Outcome {
    res.send_text("Hello, world!");
    next!()
}

fn main() {
    let mut app = App::new();
    app.get("/", hello);
    app.listen("127.0.0.1:5050");
}
```

That’s all — no async.

---

## Middleware in Feather

Middleware is the heart of Feather. You may write it as a closure (using the `middleware!` macro), a struct, or chain them together:


```rust
use feather::{App, middleware_fn, next};

#[middleware_fn]
fn log_middleware() -> feather::Outcome {
    println!("Custom global middleware!");
    next!()
}

#[middleware_fn]
fn hello() -> feather::Outcome {
    res.send_text("Hello, world!");
    next!()
}

fn main() {
    let mut app = App::new();
    app.use_middleware(log_middleware);
    app.get("/", hello);
    app.listen("127.0.0.1:5050");
}
```

Or as a struct:


```rust
use feather::{middleware_fn, Request, Response, AppContext, Middleware, next, info};""

struct CustomMiddleware(String);

impl Middleware for CustomMiddleware {
    fn handle(&self, _request: &mut Request, _response: &mut Response, _ctx: &AppContext) -> feather::Outcome {
        info!("Hii I am a Struct Middleware and this is my data: {}", self.0);
        next!()
    }
}


```

---

## State Management using the Context API

Feather's Context API allows you to manage application-wide state without extractors or macros.


```rust
use feather::{App, middleware_fn, next};
#[derive(Debug)]
struct Counter { pub count: i32 }

#[middleware_fn]
fn count() -> feather::Outcome {
    let counter = ctx.get_state::<State<Counter>>().unwrap();
    counter.lock().count += 1;
    res.send_text(format!("Counted! {}", counter.count));
    next!()
}

fn main() {
    let mut app = App::new();
    app.context().set_state(State::new(Counter { count: 0 }));
    app.get("/", count);
    app.listen("127.0.0.1:5050");
}
```

Context is especially useful when needing to access databases and files.

## Built-in JWT Authentication

Feather has a native JWT module activated using a cargo feature `jwt`:

```toml
[dependencies]
feather = { version = "*", features = ["jwt"] }
```


```rust
use feather::{App, jwt_required, middleware_fn, next};
use feather::jwt::{JwtManager, SimpleClaims};

#[middleware_fn]
fn token_route() -> feather::Outcome {
    let token = ctx.jwt().generate_simple("user", 1)?;
    res.send_text(format!("Token: {}", token));
    next!()
}

#[jwt_required]
#[middleware_fn]
fn protected(claims: SimpleClaims) -> feather::Outcome {
    res.send_text(format!("Hello, {}!", claims.sub));
    next!()
}

fn main() {
    let mut app = App::new();
    let manager = JwtManager::new("secretcode");
    app.context().set_jwt(manager);
    app.get("/token", token_route);
    app.get("/auth", protected);
    app.listen("127.0.0.1:8080");
}
```
---

## Logging
When you create a new Feather application, it initializes the logger by default.


```rust
fn main() {
    let mut app = App::new();
    info!("Feather app ready to serve requests!");
    // Your app setup here
    app.listen("127.0.1:5050");
    
}
```
if you don't want it to be initialized, you can disable it by create a App with `without_logger` method

---

## Goals

- Be the simplest Rust web framework to get started with
- Be modular and easy to extend
- Focus on DX without sacrificing Rust's safety and performance

---

## Contributing

PRs are welcome!  
If you have ideas or bugs, please [open an issue]([https://github.com/BersisSe/feather/issues) or submit a pull request.

```bash
# Getting started with dev
git clone https://github.com/BersisSe/feather.git
cd feather
cargo run --example app
```

---

## License

Feather is MIT licensed. See [LICENSE](./LICENSE).

---

## Acknowledgments

Feather is inspired by [Express.js](https://expressjs.com) and exists to bring that same productivity to Rust.  
Huge thanks to the Rust community for their support and contributions!  
Special thanks to the contributors who have helped make Feather better!    

---

## Spread the Word

If you like Feather:

- ⭐ Star it on [GitHub](https://github.com/BersisSe/feather),
- Share it on Reddit, HN, or Discord
- Build something and show up!

---
