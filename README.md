<h1 align="center">🪶 Feather</h1>

<p align="center">
  <a href="https://crates.io/crates/feather"><img src="https://img.shields.io/crates/v/feather.svg" alt="Crates.io"/></a>
  <a href="https://docs.rs/feather"><img src="https://docs.rs/feather/badge.svg" alt="Docs.rs"/></a>
  <a href="./LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"/></a>
</p>

## **Feather** is a lightweight, DX-first web framework for Rust. inspired by the simplicity of Express.js, but designed for Rust’s performance and safety.

## Why Feather?

- **Middleware-First Architecture**  
  Everything is a middleware even if they are not a middleware they produce a middleware in the end.  

- **Easy State Management Using Context**  
  Recently implemented the Context API that makes it very easy to manage state without the use of Extractors/Macros.  

- **All in One**  
  Feather is a complete web framework that includes routing, middleware, logging, JWT authentication, and more, all in one package.

- **Feel of Async Without Async**  
  Feather is Multithreaded by default running on **Feather-Runtime**.
  

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
use feather::middleware::builtins;
use feather::{App, AppContext, next};
use feather::{Request, Response};
fn main() {
    let mut app = App::new();
    app.get("/", |_request: &mut Request, response: &mut Response, _ctx: &mut AppContext| {
        response.send_text("Hello, world!");
        next!()
    });
    
    app.use_middleware(builtins::Logger);
    app.listen("127.0.0.1:5050");
}
```

That’s all — no async.

---

## Middleware in Feather

Middleware is intented to be the heart of Feather. You may write it as a closure, a struct, or chain them together:

```rust
use feather::{App, AppContext, Request, Response,next,Outcome};
use feather::middleware::builtins;
use feather::middleware::{Middleware, MiddlewareResult};

// Implementors of the Middleware trait are middleware that can be used in a Feather app.
struct Custom;

impl Middleware for Custom {
    fn handle(&self, request: &mut Request, _response: &mut Response, _ctx: &mut AppContext) -> Outcome {
      println!("Now running some custom middleware (struct Custom)!");
      println!("And there's a request with path: {:?}", request.uri);
      next!()
    }
}

fn main() {
    let mut app = App::new();
    app.use_middleware(builtins::Logger);
    app.use_middleware(Custom);
    app.use_middleware(|_req: &mut Request, _res: &mut Response, _ctx: &mut AppContext| {
        println!("Now running some custom middleware (closure)!");
        next!()
    });

    app.get("/",|_req: &mut Request, res: &mut Response, _ctx: &mut AppContext| {
        res.send_text("Hello, world!");
        next!()
    });

    app.listen("127.0.0.1:5050");
}
```
---

## State Management using the Context API

Feather's new Context API allows you to manage application-wide state without extractors or macros.

As an example:

```rust
use feather::{next, App, AppContext, Request, Response};
// Create a counter struct to hold the state
#[derive(Debug)]
struct Counter {
    pub count: i32,
}
fn main() {
    let mut app = App::new();
    let counter = Counter { count: 0 };
    app.context().set_state(counter);

    app.get("/",move |_req: &mut Request, res: &mut Response, ctx: &mut AppContext| {
      let counter: &mut Counter = ctx.get_mut_state::<Counter>().unwrap();
      counter.count += 1;
      res.send_text(format!("Counted! {}", counter.count));
      next!()
    });
    // Lastly add a route to get the current count
    app.get("/count",move |_req: &mut Request, res: &mut Response, ctx: &mut AppContext| {
      let counter = ctx.get_state::<Counter>().unwrap();
      res.send_text(counter.count.to_string());
      next!()
    });
    app.listen("127.0.0.1:5050");
}

```

Context is especially useful when needing to access databases and files.

## Built-in JWT Authentication

Feather has a native JWT module activated using a cargo feature `jwt`:

```toml
[dependencies]
feather = { version = "0.3.1", features = ["jwt"] }
```

```rust
use feather::jwt::{generate_jwt, with_jwt_auth};
use feather::{App, AppContext,next};

fn main() {
    let mut app = App::new();
    app.get("/auth",with_jwt_auth("secretcode", |_req, res,_ctx, claim| {
        println!("Claim: {:?}", claim);
        res.send_text("Hello, JWT!");
        next!()
      }),
    );
    // Check the JWT Example for a more complete version!
    app.listen("127.0.0.1:8080")
}
```

---

## Goals

- Being the simplest Rust web framework to get started with
- Being modular and easy to extend
- Focusing on DX without sacrificing Rust's safety and performance

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
