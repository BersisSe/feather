use std::borrow::Cow;
use std::sync::Arc;

use feather_runtime::Method;
use feather_runtime::http::{Request, Response};

use super::route_methods;
use crate::internals::app::Route;
use crate::middlewares::Middleware;
use crate::{AppContext, MiddlewareResult, Outcome};

/// A modular router for grouping related routes and applying scoped middleware.
///
/// `Router` allows you to build sub-sections of your application (e.g., an `/api` or `/auth` module)
/// and mount them to the main `App` later. Middleware added to a `Router` only executes for
/// routes defined within that router.
/// # Example
/// ```rust,ignore
/// let mut app = App::new();
/// let api = Router::new();
///
/// app.mount("/api", api)
/// ```
pub struct Router {
    pub(crate) routes: Vec<Route>,
    pub(crate) middleware: Vec<Arc<dyn Middleware>>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            routes: Vec::new(),
            middleware: Vec::new(),
        }
    }

    pub fn use_middleware<M: Middleware + 'static>(&mut self, mw: M) {
        self.middleware.push(Arc::new(mw));
    }

    pub fn route<M: Middleware + 'static>(&mut self, method: Method, path: impl Into<Cow<'static, str>>, mw: M) {
        self.routes.push(Route {
            method,
            path: path.into(),
            middleware: Arc::new(mw),
        });
    }

    route_methods!(
        GET get
        POST post
        PUT put
        DELETE delete
        PATCH patch
        HEAD head
        OPTIONS options
    );
}

/// This is a Light Wrapper Middleware that handles the scoping logic
pub(crate) struct ScopedMiddleware {
    pub router_stack: Vec<Arc<dyn Middleware>>,
    pub route_handler: Arc<dyn Middleware>,
}

impl Middleware for ScopedMiddleware {
    fn handle(&self, req: &mut Request, res: &mut Response, ctx: &AppContext) -> Outcome {
        // 1. Run Router-level middlewares
        for mw in &self.router_stack {
            match mw.handle(req, res, ctx)? {
                MiddlewareResult::Next => continue,
                other => return Ok(other), // End or NextRoute
            }
        }
        // 2. Finally run the actual handler
        self.route_handler.handle(req, res, ctx)
    }
}
