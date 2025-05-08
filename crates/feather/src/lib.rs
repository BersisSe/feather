#![doc = include_str!("../../../README.md")]

/// The [`Middleware`] trait and some common middleware primitives.
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