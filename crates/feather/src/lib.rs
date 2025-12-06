//! # 🪶 Feather: Middleware-First, DX-First Web Framework for Rust
//!
//! Feather is a lightweight, middleware-first web framework for Rust, inspired by the simplicity of Express.js but designed for Rust’s performance and safety.
//!
//! ## Philosophy
//! - **Middleware-First**: Everything in Feather is a middleware, or produces a middleware. This enables powerful composition and a familiar, flexible mental model.
//! - **DX-First**: Feather is designed to be easy to use, with minimal boilerplate, clear APIs, and a focus on developer experience.
//!
//! ## Features
//! - **Express.js-like Routing**: Use `app.get` style routing for simplicity and familiarity.
//! - **State Management**: Manage application state efficiently using the Context API.
//! - **Error Handling**: Runtime error handling for easier debugging and recovery.
//! - **Middleware Support**: Create and chain middlewares for modular and reusable code.
//! - **All-in-One**: Includes routing, middleware, logging, JWT authentication, and more.
//! - **Multithreaded by Default**: Powered by Feather-Runtime for high performance without async.
//!
//! ## Getting Started
//! Add Feather to your `Cargo.toml`:
//! ```toml
//! [dependencies]
//! feather = "~0.4"
//! ```
//!
//! ### Quick Example
//! ```rust,ignore
//! use feather::{App, AppContext, Request, Response, next, middleware};
//! fn main() {
//!     let mut app = App::new();
//!     app.get("/", middleware!(|_req, res, _ctx| {
//!         res.send_text("Hello, Feather!");
//!         next!()
//!     }));
//!     app.listen("127.0.0.1:5050");
//! }
//! ```
//!
//! ## Middleware in Feather
//! Middleware is the heart of Feather. You can write middleware as a closure, a struct, or chain them together. The `middleware!` macro helps reduce boilerplate for closures:
//!
//! ```rust,ignore
//! app.get("/", middleware!(|req, res, ctx| {
//!     // ...
//!     next!()
//! }));
//! ```
//!
//! ## State Management
//! Feather's Context API allows you to manage application-wide state without extractors or macros. See the README for more details.
//!
//! ## Macros
//! - `next!`: Syntactic sugar for `Ok(MiddlewareResult::Next)`, reducing boilerplate in middleware implementations.
//! - `middleware!`: Concise closure middleware definition.
//!
//! ---

pub mod internals;
#[cfg(feature = "jwt")]
pub mod jwt;

pub mod middlewares;

#[cfg(feature = "json")]
pub use serde_json::{Value, json};

#[cfg(feature = "log")]
pub use log::{info, trace, warn};

use std::error::Error;

pub use crate::middlewares::MiddlewareResult;
pub use feather_runtime::http::{Request, Response};
pub use internals::{App, AppContext};

/// This is just a type alias for `Result<MiddlewareResult, Box<dyn Error>>;`  
/// Outcome is used in All middlewares as a return type.
pub type Outcome = Result<MiddlewareResult, Box<dyn Error>>;

/// This macro is just a syntactic sugar over the `Ok(MiddlewareResult::Next)`
/// syntax just to clear some Boilerplate
#[macro_export]
macro_rules! next {
    () => {
        Ok($crate::middlewares::MiddlewareResult::Next)
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
