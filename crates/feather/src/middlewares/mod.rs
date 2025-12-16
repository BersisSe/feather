//! Middleware system for request processing.
//!
//! Feather's middleware system is the core of request handling. Everything that
//! processes HTTP requests - routes, validation, authentication, etc. - is a middleware.
//!
//! # Core Concepts
//!
//! - [`Middleware`] - Trait for implementing request handlers
//! - [`MiddlewareResult`] - Enum controlling request flow
//! - [`builtins`] - Pre-built middleware for common tasks
//!
//! # Using Middleware
//!
//! Add global middleware that runs on every request:
//!
//! ```rust,ignore
//! app.use_middleware(|req, res, ctx| {
//!     println!("Request to: {}", req.uri);
//!     Ok(MiddlewareResult::Next)
//! });
//! ```
//!
//! Add route-specific middleware:
//!
//! ```rust,ignore
//! app.get("/", |req, res, ctx| {
//!     res.send_text("Hello!");
//!     Ok(MiddlewareResult::Next)
//! });
//! ```

pub mod builtins;
pub mod common;

pub use common::{Middleware, MiddlewareResult, chain};
