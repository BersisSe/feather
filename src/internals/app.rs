use super::AppContext;
use crate::middleware::Middleware;
use colored::Colorize;
pub use feather_runtime::Method;
use feather_runtime::http::{Request, Response};
use feather_runtime::server::server::Server;
use feather_runtime::server::server::ServerConfig;
use std::borrow::Cow;
use std::{fmt::Display, net::ToSocketAddrs};

/// A route in the application.  
pub struct Route {
    method: Method,
    path: Cow<'static, str>,
    middleware: Box<dyn Middleware>,
}

/// A Feather application.  

pub struct App {
    routes: Vec<Route>,
    middleware: Vec<Box<dyn Middleware>>,
    context: AppContext,
}

macro_rules! route_methods {
    ($($method:ident $name:ident)+) => {
        $(
            /// Adds a route to the application for the HTTP method.
            pub fn $name<M: Middleware + 'static>(&mut self, path: impl Into<String>, middleware: M) {
                self.route(Method::$method, path.into(), middleware);
            }
        )+
    }
}

impl App {
    /// Create a new instance of the application
    #[must_use = "Does nothing if you don't use the `listen` method"]
    pub fn new() -> Self {
        Self {
            routes: Vec::new(),
            middleware: Vec::new(),
            context: AppContext::new(),
        }
    }
    /// Returns a Handle to the [AppContext] inside the App
    /// [AppContext] is Used for App wide state managment
    pub fn context(&mut self) -> &mut AppContext {
        &mut self.context
    }
    /// Add a route to the application.  
    /// Every Route Returns A MiddlewareResult to control the flow of your application.
    pub fn route<M: Middleware + 'static>(
        &mut self,
        method: Method,
        path: impl Into<Cow<'static, str>>,
        middleware: M,
    ) {
        self.routes.push(Route {
            method,
            path: path.into(),
            middleware: Box::new(middleware),
        });
    }

    /// Add a global middleware to the application that will be applied to all routes.
    pub fn use_middleware(&mut self, middleware: impl Middleware + 'static) {
        self.middleware.push(Box::new(middleware));
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
        global_middleware: &[Box<dyn Middleware>],
        mut context: &mut AppContext,
    ) -> Response {
        let mut response = Response::default();
        // Run global middleware
        for middleware in global_middleware {
            middleware.handle(&mut request, &mut response, &mut context);
        }
        // Run route-specific middleware
        if let Some(route) = routes
            .iter()
            .find(|r| r.method == request.method && r.path == request.uri.to_string())
        {
            route.middleware.handle(request, &mut response, &mut context);
        }
        response
    }

    /// Start the application and listen for incoming requests on the given address.
    /// Blocks the current thread until the server is stopped.
    ///
    /// # Panics
    ///
    /// Panics if the server fails to start
    pub fn listen(&mut self, address: impl ToSocketAddrs + Display) {
        let server_conf = ServerConfig {
            address: address.to_string(),
        };
        let server = Server::new(server_conf);
        println!(
            "{} : {}",
            "Feather Listening on".blue(),
            format!("http://{address}").green()
        );
        let routes = &self.routes;
        let middleware = &self.middleware;
        let mut ctx = &mut self.context;
        server
            .incoming()
            .for_each(move |mut req| {
                let response = Self::run_middleware(&mut req, &routes, &middleware, &mut ctx);
                return response;
            })
            .unwrap();
    }
}
