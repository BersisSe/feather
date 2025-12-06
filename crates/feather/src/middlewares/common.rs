use crate::{Outcome, internals::AppContext};
use feather_runtime::http::{Request, Response};

pub trait Middleware: Send + Sync {
    /// Handle the Request sycro
    fn handle(&self, request: &mut Request, response: &mut Response, ctx: &AppContext) -> Outcome;
}

#[derive(Debug)]
pub enum MiddlewareResult {
    /// Continue to the next middleware.
    Next,
    /// Skip all subsequent middleware and continue to the next route.
    NextRoute,
}

/// Implement the `Middleware` trait for a slice of middleware.
impl Middleware for [&Box<dyn Middleware>]
where
    Self: Send + Sync,
{
    fn handle(&self, request: &mut Request, response: &mut Response, ctx: &AppContext) -> Outcome {
        for middleware in self {
            if matches!(middleware.handle(request, response, ctx), Ok(MiddlewareResult::NextRoute)) {
                return Ok(MiddlewareResult::NextRoute);
            }
        }
        Ok(MiddlewareResult::Next)
    }
}

///Implement the `Middleware` trait for Closures with Request and Response Parameters.
impl<F> Middleware for F
where
    F: Fn(&mut Request, &mut Response, &AppContext) -> Outcome + Send + Sync,
{
    fn handle(&self, req: &mut Request, res: &mut Response, ctx: &AppContext) -> Outcome {
        self(req, res, ctx)
    }
}
/// Can be used to chain two middlewares together.
/// The first middleware will be executed first.
/// If it returns `MiddlewareResult::Next`, the second middleware will be executed.
pub fn _chainer<A, B>(a: A, b: B) -> impl Middleware
where
    A: Middleware,
    B: Middleware,
{
    move |request: &mut Request, response: &mut Response, ctx: &AppContext| -> Outcome {
        match a.handle(request, response, ctx) {
            Ok(MiddlewareResult::Next) => b.handle(request, response, ctx),
            Ok(MiddlewareResult::NextRoute) => Ok(MiddlewareResult::NextRoute),
            Err(e) => Err(e),
        }
    }
}

#[macro_export]
/// A macro to chain multiple middlewares together.<br>
/// This macro takes a list of middlewares and chains them together.
macro_rules! chain {
    ($first:expr, $($rest:expr),+ $(,)?) => {{
        let chained = $first;
        $(let chained = $crate::middlewares::common::_chainer(chained, $rest);)+
        chained
    }};
}
pub use chain;
