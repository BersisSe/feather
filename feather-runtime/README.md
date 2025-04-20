# Feather Runtime

**Feather Runtime** is a lightweight, multithreaded HTTP server engine for [Feather](https://github.com/BersisSe/feather#). It provides enhanced control over low-level server operations and replaces `tiny-http`, which is no longer maintained.

## Features

- 🚀 **Multithreaded Request Handling** – Efficiently handles multiple connections using a thread pool.
- 🔄 **Graceful Shutdown** – Cleanly shuts down on signal
- 🌍 **Dynamic HTTP Responses** – Easily generate responses based on incoming requests.
- ⚡ **Buffered I/O** – Optimized performance with `BufReader` and `BufWriter`.


## Why Feather Runtime?

I built **Feather Runtime** because I wanted greater control over the low-level aspects of the framework. `tiny-http`, the library Feather was originally based on, is no longer maintained, making it necessary to develop a more flexible and future-proof solution.

Feather Runtime ensures a modern, efficient, and reliable HTTP server experience tailored for Feather.

### For Contributors

If you're contributing to Feather but don't want to mess with low-level server internals, you can mostly ignore this subcrate. Feather Runtime is designed to handle the core HTTP processing while Feather itself provides higher-level abstractions. If you have a feature request or a problem open a [Issue](https://github.com/BersisSe/feather/issues) for it 

---

