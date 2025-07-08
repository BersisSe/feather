# Feather Runtime

Hi! Feather Runtime is the engine that powers [Feather](https://github.com/BersisSe/feather#). I built it because I wanted a web server that feels as simple as writing synchronous Rust, but can still handle thousands of connections without breaking a sweat. If you’re tired of fighting with async/await or just want to see how far coroutines can take you in Rust, you’re in the right place.

It replaces `tiny-http` with a modern, coroutine-based runtime—no async/await required.

---



## 🚀 Features 

- **Coroutines, Not Threads:** Thanks to the [`may`](https://github.com/Xudong-Huang/may) crate, every connection gets its own coroutine (a green thread). This means you can handle a ton of traffic without your server falling over.
- **Non-blocking I/O:** All sockets are non-blocking, so the server stays snappy even when things get busy.
- **No async/await Headaches:** Just write normal Rust code. Feather-Runtime takes care of the scheduling magic behind the scenes.
- **Graceful Shutdown:** Hit Ctrl+C and the server shuts down cleanly, no weird hangs.
- **Dynamic HTTP Responses:** Build and send responses however you want—no fuss.
- **Buffered I/O:** Uses `BufReader` and `BufWriter` for speed.
- **Custom Socket Tuning:** Want to tweak backlog, nodelay, or buffer sizes? It’s all there via [`socket2`](https://docs.rs/socket2).
- **Extensible:** It’s the engine for Feather, but you can use it for your own experiments too.

---

## 🛠️ How It Works

I wanted something that “just works” for high concurrency, but doesn’t make you write async spaghetti. Here’s the gist:

- **Coroutines, Not Threads:** Instead of a thread per connection, every connection gets a coroutine (thanks, `may`). Coroutines are super lightweight, so you can have thousands running at once.
- **Non-blocking, Event-driven:** Sockets are non-blocking. When a request comes in, it’s handed to a coroutine. If that coroutine needs to wait for I/O, it just yields and lets others do their thing. No wasted CPU, no blocking the whole server.
- **Message Queues:** Requests go into a queue, and coroutines pick them up, process, and respond. This keeps things smooth and scalable.
- **No async/await, No Lifetimes:** Write normal Rust. No async, no lifetimes, no pinning. The runtime handles all the tricky stuff.
- **Socket Tuning:** Want to tweak how the server listens? Use `socket2` to set backlog, nodelay, buffer sizes, etc.

**In summary:**
> Every request gets its own coroutine. You can spawn background tasks or run blocking code, and Feather-Runtime will keep things fast and responsive—no async/await or lifetime headaches.

---

## 📦 Example Usage

Here’s a minimal example of using Feather-Runtime directly (normally, you use it via Feather):

```rust
use feather_runtime::runtime::engine::Engine;
use feather_runtime::http::{Request, Response};

fn main() {
    let engine = Engine::new("127.0.0.1:5050");
    engine.start();
    engine.for_each(|req: &mut Request| {
        let mut res = Response::default();
        res.send_text("Hello from Feather-Runtime!");
        res
    }).unwrap();
}
```

---

## 🤝 For Contributors

If you're contributing to Feather but don't want to mess with low-level server internals, you can mostly ignore this subcrate. Feather-Runtime is designed to handle the core HTTP processing while Feather itself provides higher-level abstractions. If you have a feature request or a problem, open an [issue](https://github.com/BersisSe/feather/issues).

---

## 📚 Learn More

- [Feather Main Repo](https://github.com/BersisSe/feather)
- [`may` Crate](https://github.com/Xudong-Huang/may)
- [`socket2` Crate](https://docs.rs/socket2)

---

## License

Feather-Runtime is MIT licensed. See [LICENSE](../LICENSE).

