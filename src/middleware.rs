use crate::response::Response;
use dyn_clone::DynClone;
use tiny_http::Request;

/// Common trait for all middleware types. Implemented automatically for functions fitting
/// the `(request, response) -> result` signature.
pub trait Middleware: Send + Sync + DynClone {
    /// Handle an incoming request by transforming it into a response.
    fn handle(&self, request: &mut Request, response: &mut Response) -> MiddlewareResult;
}

dyn_clone::clone_trait_object!(Middleware);

pub enum MiddlewareResult {
    /// Continue to the next middleware.
    Next,
    /// Skip all subsequent middleware and continue to the next route.
    NextRoute,
}

#[derive(Clone)]
/// Log incoming requests and transparently pass them to the next middleware.
pub struct Logger;

impl Middleware for Logger {
    fn handle(&self, request: &mut Request, _: &mut Response) -> MiddlewareResult {
        println!("Request: {request:?}");
        MiddlewareResult::Next
    }
}

#[derive(Clone, Default)]
/// Add [CORS] headers to the response.
///
/// [CORS]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Guides/CORS
pub struct Cors(Option<String>);

impl Cors {
    #[must_use]
    pub const fn new(origin: String) -> Self {
        Self(Some(origin))
    }
}

impl Middleware for Cors {
    fn handle(&self, _: &mut Request, response: &mut Response) -> MiddlewareResult {
        response.with_header(
            "Access-Control-Allow-Origin",
            self.0.as_deref().unwrap_or("*"),
        );
        MiddlewareResult::Next
    }
}

impl Middleware for [&Box<dyn Middleware>] {
    fn handle(&self, request: &mut Request, response: &mut Response) -> MiddlewareResult {
        for middleware in self {
            if matches!(
                middleware.handle(request, response),
                MiddlewareResult::NextRoute
            ) {
                return MiddlewareResult::NextRoute;
            }
        }
        MiddlewareResult::Next
    }
}

impl<F: Fn(&mut Request, &mut Response) -> MiddlewareResult + Sync + Send + DynClone> Middleware
    for F
{
    fn handle(&self, request: &mut Request, response: &mut Response) -> MiddlewareResult {
        self(request, response)
    }
}
