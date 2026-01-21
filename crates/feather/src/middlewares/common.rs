use crate::{Outcome, internals::AppContext};
use feather_runtime::http::{Request, Response};

/// Core middleware trait for request processing.
///
/// Every request handler in Feather is a middleware. Implement this trait to create
/// custom request processors.
///
/// # Example
///
/// ```rust,ignore
/// use feather::Middleware;
///
/// struct LoggingMiddleware;
///
/// impl Middleware for LoggingMiddleware {
///     fn handle(&self, request: &mut Request, response: &mut Response, ctx: &AppContext) -> Outcome {
///         println!("{} {}", request.method, request.uri);
///         next!()
///     }
/// }
/// ```
pub trait Middleware: Send + Sync {
    /// Handle the incoming request and return the result.
    ///
    /// This method is called for each incoming HTTP request. You can:
    /// - Read the request (headers, body, path)
    /// - Modify the response (headers, body, status code)
    /// - Access application state via `ctx`
    /// - Control flow with the return value
    fn handle(&self, request: &mut Request, response: &mut Response, ctx: &AppContext) -> Outcome;
}

#[derive(Debug)]
pub enum MiddlewareResult {
    /// Continue to the next middleware in the chain.
    Next,
    /// Skip remaining middleware and move to the next route handler.
    ///
    /// Use this to abort the current request processing and prevent subsequent
    /// middleware from executing.
    NextRoute,
    /// Response is handled send it now.
    End,
}

/// Implement the `Middleware` trait for a slice of middleware.
impl Middleware for [&Box<dyn Middleware>]
where
    Self: Send + Sync,
{
    fn handle(&self, request: &mut Request, response: &mut Response, ctx: &AppContext) -> Outcome {
        for middleware in self {
            let res = middleware.handle(request, response, ctx)?;
            match res {
                MiddlewareResult::Next => continue,
                MiddlewareResult::NextRoute => return Ok(MiddlewareResult::NextRoute),
                MiddlewareResult::End => return Ok(MiddlewareResult::End),
            }
        }
        Ok(MiddlewareResult::Next)
    }
}

/// Automatically implement `Middleware` for function closures.
///
/// This allows you to use plain closures as middleware without wrapping them:
///
/// ```rust,ignore
/// app.use_middleware(|req, res, ctx| {
///     // Your middleware logic
///     Ok(MiddlewareResult::Next)
/// });
/// ```
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
///
/// # Example
///
/// ```rust,ignore
/// let middleware1 = |req: &mut Request, res: &mut Response, ctx: &AppContext| {
///     Ok(MiddlewareResult::Next)
/// };
///
/// let middleware2 = |req: &mut Request, res: &mut Response, ctx: &AppContext| {
///     Ok(MiddlewareResult::Next)
/// };
///
/// let chained = chain(middleware1, middleware2);
/// app.use_middleware(chained);
/// ```
pub fn _chainer<A, B>(a: A, b: B) -> impl Middleware
where
    A: Middleware,
    B: Middleware,
{
    move |request: &mut Request, response: &mut Response, ctx: &AppContext| -> Outcome {
        match a.handle(request, response, ctx) {
            Ok(MiddlewareResult::Next) => b.handle(request, response, ctx),
            Ok(MiddlewareResult::NextRoute) => Ok(MiddlewareResult::NextRoute),
            Ok(MiddlewareResult::End) => Ok(MiddlewareResult::End), 
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
