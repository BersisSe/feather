#![doc = include_str!("../README.md")]

/// The [`Middleware`] trait and some common middleware primitives.
pub mod middleware;
/// The [`Response`] type.
mod response;
/// Synchronous API for Feather.
mod sync;

pub use crate::middleware::Middleware;
pub use crate::sync::AppConfig;
pub use response::Response;
pub use sync::App;
pub use tiny_http::Request;
