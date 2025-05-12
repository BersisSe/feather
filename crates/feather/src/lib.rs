//! Feather is a lightweight and extensible web framework for Rust, inspired by the simplicity of Express.js.
//!
//! # Features
//! - **Express.js-like Routing**: Use `app.get` style routing for simplicity and familiarity.
//! - **State Management**: Manage application state efficiently using the Context API.
//! - **Error Handling**: Runtime error handling for easier debugging and recovery.
//! - **Middleware Support**: Create and chain middlewares for modular and reusable code.
//! - **Out of the Box Support**: Things like JWT are supported via Cargo Features
//!
//!
//! ## Example
//! ```rust,no_run
//! use feather::{App, Request, Response, AppContext, next};
//!
//! fn main() {
//!     let mut app = App::new();
//! 
//!     app.get("/", |_req: &mut Request, res: &mut Response, _ctx: &mut AppContext| {
//!         res.send_text("Hello, Feather!");
//!         next!()
//!     });
//! 
//!     app.listen("127.0.0.1:5050");
//! }
//! ```
//!
//! # Modules
//! - `middleware`: Define and use custom middlewares.
//! - `internals`: Core components like `App` and `AppContext`.
//! - `jwt` (optional): JWT utilities for authentication (requires the `jwt` feature).
//!
//! # Type Aliases
//! - `Outcome`: A type alias for `Result<MiddlewareResult, Box<dyn Error>>`, used as the return type for middlewares.
//!
//! # Macros
//! - `next!`: A syntactic sugar for `Ok(MiddlewareResult::Next)`, reducing boilerplate in middleware implementations.

pub mod middleware;
pub mod internals;
#[cfg(feature = "jwt")]
pub mod jwt;

use std::error::Error;

pub use feather_runtime::http::{Request,Response};
pub use internals::{App, AppContext};
pub use crate::middleware::MiddlewareResult;

/// This is just a type alias for `Result<MiddlewareResult, Box<dyn Error>>;`  
/// Outcome is used in All middlewares as a return type.
pub type Outcome = Result<MiddlewareResult, Box<dyn Error>>;

/// This macro is just a syntactic sugar over the `Ok(MiddlewareResult::Next)`
/// syntax just to clear some Boilerplate
#[macro_export]
macro_rules! next {
    () => {
        Ok($crate::middleware::MiddlewareResult::Next)
    };
}
