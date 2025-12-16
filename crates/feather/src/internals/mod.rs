//! Internal implementation details of the Feather framework.
//!
//! This module contains the core application logic, state management, and error handling.
//! Most users will only interact with [`App`] and [`AppContext`].

mod app;
mod context;
pub use app::App;
pub use context::AppContext;
mod error_stack;
mod service;
pub use context::State;
pub use feather_runtime::{HeaderMap, HeaderName, HeaderValue, Method, Uri};
