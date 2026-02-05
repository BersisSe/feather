//! Internal implementation details of the Feather framework.
//!
//! This module contains the core application logic, state management, and error handling.
//! Most users will only interact with [`App`] and [`AppContext`].

mod app;
mod context;
mod error_stack;
mod router;
mod runtime_extensions;
mod service;

pub use app::App;
pub use context::AppContext;
pub use context::State;
pub use feather_runtime::{HeaderMap, HeaderName, HeaderValue, Method, Uri};
pub use router::Router;
pub use runtime_extensions::Finalizer;

/// Used internally to generate the route methods for DRY(Don't Repeat Yourself).
macro_rules! route_methods {
    ($($method:ident $name:ident)+) => {
        $(
            /// Adds a route to the application for the HTTP method.
            #[inline]
            pub fn $name<M: Middleware + 'static>(&mut self, path: impl Into<String>, middleware: M) {
                self.route(Method::$method, path.into(), middleware);
            }
        )+
    }
}

use route_methods;
