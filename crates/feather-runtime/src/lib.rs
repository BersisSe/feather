//! # Feather Runtime
//!
//! Low-level runtime and HTTP primitives for the Feather web framework.
//!
//! Feather-Runtime provides:
//! - Synchronous HTTP request/response handling
//! - Lightweight coroutine-based concurrency using the `may` crate
//! - Zero-copy networking for high performance
//! - Thread-safe server implementation
//!
//! ## Core Components
//!
//! - [`http`] - HTTP request and response types
//! - [`runtime`] - Server runtime and coroutine support

pub mod http;
pub mod runtime;

pub use ::http::{HeaderMap, HeaderName, HeaderValue, Method, Uri};
