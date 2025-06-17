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
  

## How it works behind the scenes:  
Every request is given a thread from the server's threadpool and that thread is responsible for returning the response to that request.  
So you can run long running tasks on another thread in the middlewares, but the response can only be returned from the middleware the request is accepted on.  
If you want to go deeper take a look at [Feather-Runtime](./crates/feather-runtime)  

---

## Getting Started

Add Feather to your `Cargo.toml`:

```toml
[dependencies]
feather = "~0.4"
```

---

## Quick Example

```rust
use feather::middlewares::builtins;
use feather::{App, next, middleware};
fn main() {
    let mut app = App::new();
    app.get("/", middleware!(|_req, res, _ctx| {
        res.send_text("Hello, world!");
        next!()
    }));
    app.use_middleware(builtins::Logger);
    app.listen("127.0.0.1:5050");
}
```

That’s all — no async.

---

## Middleware in Feather

Middleware is the heart of Feather. You may write it as a closure (using the `middleware!` macro), a struct, or chain them together:

```rust
use feather::{App, next, middleware};

fn main() {
    let mut app = App::new();
    app.use_middleware(middleware!(|_req, _res, _ctx| {
        println!("Custom global middleware!");
        next!()
    }));
    app.get("/", middleware!(|_req, res, _ctx| {
        res.send_text("Hello, world!");
        next!()
    }));
    app.listen("127.0.0.1:5050");
}
```

Or as a struct:

```rust
use feather::{middlewares::Middleware, next};
struct Custom;
impl Middleware for Custom {
    fn handle(&self, req: &mut feather::Request, res: &mut feather::Response, ctx: &mut feather::AppContext) -> feather::Outcome {
        println!("Custom struct middleware!");
        next!()
    }
}
```

---

## State Management using the Context API

Feather's Context API allows you to manage application-wide state without extractors or macros.

```rust
use feather::{next, App, middleware};
#[derive(Debug)]
struct Counter { pub count: i32 }
fn main() {
    let mut app = App::new();
    app.context().set_state(Counter { count: 0 });
    app.get("/", middleware!(|_req, res, ctx| {
        let counter = ctx.get_mut_state::<Counter>().unwrap();
        counter.count += 1;
        res.send_text(format!("Counted! {}", counter.count));
        next!()
    }));
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
use feather::jwt::{generate_jwt, with_jwt_auth};
use feather::{App, next};
fn main() {
    let mut app = App::new();
    app.get("/auth", with_jwt_auth("secretcode", |_req, res, _ctx, claim| {
        println!("Claim: {:?}", claim);
        res.send_text("Hello, JWT!");
        next!()
    }));
    app.listen("127.0.0.1:8080")
}
```

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
