use dyn_clone::DynClone;
use feather_runtime::http::{Request, Response};

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

/// Implement the `Middleware` trait for a slice of middleware.
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

///Implement the `Middleware` trait for Closures with Request and Response Parameters.
impl<F: Fn(&mut Request, &mut Response) -> MiddlewareResult + Sync + Send + DynClone> Middleware
    for F
{
    fn handle(&self, request: &mut Request, response: &mut Response) -> MiddlewareResult {
        self(request, response)
    }
}

/// Can be used to chain two middlewares together.
/// The first middleware will be executed first.
/// If it returns `MiddlewareResult::Next`, the second middleware will be executed.
fn chainer<A, B>(a: A, b: B) -> impl Middleware// Nvm the warning this is used in the macro
where
    A: Middleware + Clone,
    B: Middleware + Clone,
{
    move |request: &mut Request, response: &mut Response| -> MiddlewareResult {
        match a.handle(request, response) {
            MiddlewareResult::Next => b.handle(request, response),
            MiddlewareResult::NextRoute => MiddlewareResult::NextRoute,
        }
    }
}

#[macro_export]
/// A macro to chain multiple middlewares together.<br>
/// This macro takes a list of middlewares and chains them together.
macro_rules! chain {
    ($first:expr, $($rest:expr),+ $(,)?) => {{
        let chained = $first;
        $(let chained = $crate::chainer(chained, $rest);)+
        chained
    }};
}
pub use chain;
