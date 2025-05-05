#![doc = include_str!("../../../README.md")]

/// The [`Middleware`] trait and some common middleware primitives.
pub mod middleware;

/// Synchronous API for Feather.
mod internals;
#[cfg(feature = "jwt")]
pub mod jwt;

pub use feather_runtime::http::Request;
pub use feather_runtime::http::Response;
pub use internals::{App, AppContext};
pub use middleware::MiddlewareResult;
