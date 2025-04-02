#![doc = include_str!("../README.md")]

/// The [`Middleware`] trait and some common middleware primitives.
pub mod middleware;

/// Synchronous API for Feather.
mod sync;

pub use crate::middleware::Middleware;
pub use crate::sync::AppConfig;
pub use feather_runtime::http::{HttpRequest,HttpResponse};
pub use sync::App;

