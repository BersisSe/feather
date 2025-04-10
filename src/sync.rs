use crate::middleware::Middleware;
use colored::Colorize;
use parking_lot::RwLock;
use std::{fmt::Display, net::ToSocketAddrs, sync::Arc};
use feather_runtime::Method;
use feather_runtime::http::HttpResponse as Response;
use feather_runtime::{http::HttpRequest as Request, server::tcp::Server};
/// Configuration settings for the application.
///
/// This struct is used to configure various aspects of the application,
/// such as the number of threads to be used in the thread pool.
///
/// # Fields
///
/// * `threads` - The number of threads to be used by the application's thread pool.
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// The number of threads to be used by the application's thread pool.
    pub threads: usize,
}

/// A route in the application.
#[derive(Clone)]
pub struct Route {
    method: Method,
    path: String,
    middleware: Box<dyn Middleware>,
}

/// A Feather application.
pub struct App {
    config: AppConfig,
    routes: Arc<RwLock<Vec<Route>>>,
    middleware: Arc<RwLock<Vec<Box<dyn Middleware>>>>,
}

macro_rules! route_methods {
    ($($method:ident $name:ident)+) => {
        $(
            /// Every Route takes a handler and every handler Returns a `MiddlewareResult` to control the flow of your application
            pub fn $name<M: Middleware + 'static>(&mut self, path: impl Into<String>, middleware: M)
            {
                self.route(Method::$method, path.into(), middleware);
            }
        )+
    }
}

impl App {
    /// Create a new instance of the application with default configuration.
    #[must_use = "Does nothing if you don't use the `listen` method"]
    pub fn new() -> Self {
        Self {
            config: AppConfig { threads: 6 },
            routes: Arc::new(RwLock::new(Vec::new())),
            middleware: Arc::new(RwLock::new(Vec::new())),
        }
    }
    /// Create a new instance of the application with the given configuration.
    pub fn with_config(config: AppConfig) -> Self {
        Self {
            config,
            routes: Arc::new(RwLock::new(Vec::new())),
            middleware: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Add a route to the application.
    /// 
    /// Every Route Returns A MiddlewareResult to control the flow of your application.
    /// 
    /// # Panics
    ///
    /// Panics if the internal [`RwLock`] protecting the routes is poisoned.
    pub fn route<M: Middleware + 'static>(&mut self, method: Method, path: String, middleware: M) {
        self.routes.write().push(Route {
            method,
            path,
            middleware: Box::new(middleware),
        });
    }

    /// Add a global middleware to the application that will be applied to all routes.
    ///
    /// # Panics
    ///
    /// Panics if the internal [`RwLock`] protecting the middleware is poisoned.
    pub fn use_middleware(&mut self, middleware: impl Middleware + 'static) {
        self.middleware.write().push(Box::new(middleware));
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

    fn run_middleware(
        mut request: &mut Request,
        routes: &[Route],
        middleware: &[Box<dyn Middleware>],
    ) -> Response {
        let mut response = Response::default();
        for middleware in middleware {
            middleware.handle(&mut request, &mut response);
        }
        for Route {
            method,
            path,
            middleware,
        } in routes
        {
            if *method != request.method || *path != request.uri.to_string() {
                continue;
            }
            middleware.handle(request, &mut response);
        }
        response
    }

    /// Start the application and listen for incoming requests on the given address.
    /// Blocks the current thread until the server is stopped.
    ///
    /// # Panics
    ///
    /// Panics if the server fails to start or if the internal [`RwLock`]s protecting the routes
    /// or middleware are poisoned.
    pub fn listen(&self, address: impl ToSocketAddrs + Display) {
        let server = Server::new(address.to_string(), self.config.threads);
        println!("{} : {}", "Feather Listening on".blue(),format!("http://{address}").green());
        let routes = self.routes.read().clone(); // Clone once
        let middleware = self.middleware.read().clone(); // Clone once
        server.incoming().for_each(move |mut req| {
            let response = Self::run_middleware(&mut req, &routes, &middleware);
            return response;
        }).unwrap();
    }
}
