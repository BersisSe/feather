# 🪶 Feather

**Feather** is a lightweight, DX-first web framework for Rust — inspired by the simplicity of Express.js, but designed for Rust’s performance and safety.

> 🧠 No async.  
> 🧱 Middleware-first.  
> ⚡ Just works.

---

## ✨ Why Feather?

- 🧱 **Middleware-First Architecture**  
  Everything is a middleware — route handlers, auth, logging, error handling — all composable and clean.

- 🪶 **Lightweight and Fast**  
  Feather uses traditional threads instead of async, avoiding the overhead and complexity of Rust’s async model.

- 🧑‍💻 **Developer Experience First**  
  Feather’s API is minimal, ergonomic, and readable — no lifetimes, no `.await`, no boilerplate.

- 📦 **Modular and Extensible**  
  Build the framework you want with plug-and-play middleware, simple traits, and clear primitives.

---

## 🚀 Getting Started

Add Feather to your `Cargo.toml`:

```toml
[dependencies]
feather = "0.2.0"
```

---

## 🧭 Quick Example

```rust,no_run
use feather::{App, Request, Response, MiddlewareResult};

fn main() {
    let mut app = App::new();

    app.get("/", |_req: &mut Request, res: &mut Response| {
        res.send_text("Hello, world!");
        MiddlewareResult::Next
    });

    app.listen("127.0.0.1:3000");
}
```

✔️ That’s all — no async, no magic.

---

## 🔌 Middleware in Feather

Middleware is the heart of Feather. Write it as a closure, a struct, or chain them together:

```rust,no_run
use feather::{App, Request, Response, Middleware, MiddlewareResult};

#[derive(Clone)]
struct Logger;

impl Middleware for Logger {
    fn handle(&self, req: &mut Request, _res: &mut Response) -> MiddlewareResult {
        println!("Incoming request: {}", req.uri);
        MiddlewareResult::Next
    }
}

fn main() {
    let mut app = App::new();

    app.use_middleware(Logger);
    app.use_middleware(|_req, _res| {
        println!("Inline middleware runs too!");
        MiddlewareResult::Next
    });

    app.get("/", |_req, res| {
        res.send_text("Feather is fast!");
        MiddlewareResult::Next
    });

    app.listen("127.0.0.1:3000");
}
```

---

## 🔐 Built-in JWT Authentication

Feather has native JWT middleware:

```rust,no_run
use feather::jwt::{generate_jwt, with_jwt_auth};

app.get("/auth", with_jwt_auth("secret", |req, res, claims| {
    res.send_text(format!("Hello, {}!", claims.sub));
    MiddlewareResult::Next
}));
```

---

## 🧱 Goals

- 🪶 Be the simplest Rust web framework to get started with
- 🧩 Be modular and easy to extend
- 💡 Focus on DX without sacrificing Rust's safety and performance

---

## 🤝 Contributing

PRs welcome!  
If you’ve got ideas or bugs, [open an issue](https://github.com/your_repo_link/issues) or submit a pull request.

```bash
# Getting started with dev
git clone https://github.com/your_repo_link
cd feather
cargo run --example basic
```

---

## 📄 License

Feather is MIT licensed. See [LICENSE](./LICENSE).

---

## 🙏 Acknowledgments

Feather is inspired by [Express.js](https://expressjs.com) and exists to bring that same productivity to Rust.

Thanks to the Rust community for the incredible ecosystem this project builds on.

---

## 📣 Spread the Word

If you like Feather:
- ⭐ Star it on [GitHub](https://github.com/your_repo_link)
- 📰 Share it on Reddit, HN, or Discord
- 🛠 Build something and show us!

---
