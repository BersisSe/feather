#![doc = include_str!("../README.md")]

/// The [`Middleware`] trait and some common middleware primitives.
pub mod middleware;

#[cfg(feature = "jwt")]
pub mod jwt;
/// Synchronous API for Feather.
mod sync;

pub use feather_runtime::http::Request;
pub use feather_runtime::http::Response;
pub use middleware::MiddlewareResult;
pub use sync::App;
