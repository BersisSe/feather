//! # 🪶 Feather: Synchronous DX-First Minimal Web Framework for Rust
//!
//! Feather is a lightweight, middleware-first web framework for Rust, inspired by Express.js but
//! designed for Rust's performance and safety. Build fast, synchronous web applications without async complexity.
//!
//! ## Quick Start
//!
//! Add to `Cargo.toml`:
//! ```toml
//! [dependencies]
//! feather = "0.8"
//! ```
//!
//! Hello world in 7 lines:
//! ```rust,ignore
//! use feather::prelude::*
//! fn main(){
//!     let mut app = App::new();
//!     app.get("/", middleware!(|_req, res, _ctx| {
//!         res.finish_text("Hello, Feather!")
//!     }));
//!     app.listen("127.0.0.1:5050");
//! }
//!
//! ```
//!
//! ## Why Feather?
//!
//! - **Fully Synchronous**: No async/await complexity. Built on lightweight coroutines for excellent performance.
//! - **Express.js Inspired**: Familiar API with `app.get()`, `app.post()`, middleware chains.
//! - **DX First**: Minimal boilerplate, clear APIs, easy to learn and use.
//! - **Built-in Features**: Routing, middleware, state management, error handling, JWT auth.
//! - **Multithreaded by Default**: Powered by Feather-Runtime for high concurrency.
//!
//! ## Comprehensive Guides
//!
//! Feather comes with detailed guides for every aspect:
//!
//! - **[Getting Started](guides::getting_started)** - Setup and core concepts
//! - **[Routing](guides::routing)** - HTTP methods, paths, and handlers
//! - **[Middlewares](guides::middlewares)** - Request processing pipelines
//! - **[State Management](guides::state_management)** - Application context and data sharing
//! - **[Error Handling](guides::error_handling)** - Error patterns and recovery
//! - **[Authentication](guides::authentication)** - JWT tokens and protected routes
//! - **[Server Configuration](guides::server_configuration)** - Tuning and optimization
//!
//! ## Common Tasks
//!
//! **Add a route:**
//! ```rust,ignore
//! app.get("/users/:id", middleware!(|req, res, ctx| {
//!     // Handle request
//!     res.send_text("User details");
//!     next!()
//! }));
//! ```
//!
//! **Use middleware:**
//! ```rust,ignore
//! app.use_middleware(middleware!(|req, res, _ctx| {
//!     println!("{} {}", req.method, req.uri);
//!     next!()
//! }));
//! ```
//!
//! **Manage state:**
//! ```rust,ignore
//! use feather::State;
//! app.context().set_state(State::new(MyConfig { /* ... */ }));
//! // Later in middleware:
//! let config = ctx.get_state::<State<MyConfig>>();
//! ```
//!
//! ## Next Steps
//!
//! Start with the **[Getting Started Guide](guides::getting_started)** for a comprehensive introduction,
//! or jump to any specific guide above for deep dives into features you need.
//!
//! ## Missing Feature?  
//!
//! Don't see something you need? Check out the GitHub repository for issues, feature requests, and contribution guidelines.
//! Don't hesitate to open an issue or submit a pull request!
//!
//! ---

// --- IMPORTS START ---

pub mod internals;
#[cfg(feature = "jwt")]
pub mod jwt;

pub mod middlewares;

/// Comprehensive guides and tutorials for Feather.
///
/// This module contains detailed guides for various aspects of the Feather framework,
/// including routing, middleware, state management, and more.
pub mod guides {
    /// Quick start guide to get up and running with Feather.
    ///
    #[doc = include_str!("docs/getting-started.md")]
    pub mod getting_started {}

    /// Complete guide to HTTP routing in Feather.
    ///
    #[doc = include_str!("docs/routing.md")]
    pub mod routing {}

    /// Deep dive into the middleware system.
    ///
    #[doc = include_str!("docs/middlewares.md")]
    pub mod middlewares {}

    /// Application-wide state management with AppContext.
    ///
    #[doc = include_str!("docs/state-management.md")]
    pub mod state_management {}

    /// Error handling patterns and best practices.
    ///
    #[doc = include_str!("docs/error-handling.md")]
    pub mod error_handling {}

    /// JWT authentication and protected routes.
    ///
    #[doc = include_str!("docs/authentication.md")]
    pub mod authentication {}

    /// Server configuration and performance tuning.
    ///
    #[doc = include_str!("docs/server-configuration.md")]
    pub mod server_configuration {}
}

#[cfg(feature = "json")]
pub use serde_json::{Value, json};

#[cfg(feature = "log")]
pub use log::{info, trace, warn};

use std::error::Error;

pub use crate::internals::State;
pub use crate::middlewares::MiddlewareResult;
pub use crate::middlewares::builtins;
pub use feather_runtime::http::{Request, Response};
pub use feather_runtime::runtime::server::ServerConfig;
pub use internals::{App, AppContext, Finalizer, Router};

pub mod prelude {
    pub use crate::Outcome;
    pub use crate::Request;
    pub use crate::Response;
    pub use crate::ServerConfig;
    pub use crate::State;
    pub use crate::internals::{App, AppContext, Finalizer, Router};
    pub use crate::middleware;
    pub use crate::middleware_fn;
    pub use crate::next;
}
// --- IMPORTS END ---

/// This is just a type alias for `Result<MiddlewareResult, Box<dyn Error>>;`  
/// Outcome is used in All middlewares as a return type.
pub type Outcome = Result<MiddlewareResult, Box<dyn Error>>;

/// This macro is just a syntactic sugar over the `Ok(MiddlewareResult::Next)`
///
/// **Behavior**: Continues execution to the next middleware in the current chain.
/// If used in the last middleware of the global chain, the engine proceeds to route matching.
#[macro_export]
macro_rules! next {
    () => {
        Ok($crate::middlewares::MiddlewareResult::Next)
    };
}
/// This macro is just a syntactic sugar over the `Ok(MiddlewareResult::NextRoute)`
///
/// **Behavior**: Skips the current middleware stack or route handler.
/// - In Global Middleware: Jumps straight to the Routing phase.
/// - In a Route: Skips to the next matching route (useful conditional routing).
#[macro_export]
macro_rules! next_route {
    () => {
        Ok($crate::middlewares::MiddlewareResult::NextRoute)
    };
}
/// This macro is just a syntactic sugar over the `Ok(MiddlewareResult::End)`
///
/// **Behavior**: Instantly halts all further processing (skipping remaining
/// middleware and routing) and sends the current state of the `Response` to the client.<br>
/// **Warning**: Ensure you have populated the `Response` (status, body, etc.) before
/// calling `end!`. Otherwise it will send a empty Response with a 200 code.
#[macro_export]
macro_rules! end {
    () => {
        Ok($crate::middlewares::MiddlewareResult::End)
    };
}
/// The `middleware!` macro allows you to define middleware functions concisely without repeating type signatures.
///
/// # Usage
///
/// Use the argument form to access request, response, and context objects:
///
/// ```rust,ignore
/// app.get("/", middleware!(|req, res, ctx| {
///     res.send_text("Hello, world!");
///     next!()
/// }));
/// ```
///
/// This macro expands to a closure with the correct types for Feather's middleware system.
#[macro_export]
macro_rules! middleware {
    // Argument form: middleware!(|req, res, ctx| { ... })
    (|$req:ident, $res:ident, $ctx:ident| $body:block) => {
        |$req: &mut $crate::Request, $res: &mut $crate::Response, $ctx: &$crate::AppContext| $body
    };
}

pub use feather_macros::middleware_fn;

#[cfg(feature = "jwt")]
pub use feather_macros::Claim;
#[cfg(feature = "jwt")]
pub use feather_macros::jwt_required;
