use super::AppContext;
use super::error_stack::ErrorHandler;
use crate::middlewares::Middleware;
pub use feather_runtime::Method;
use feather_runtime::http::{Request, Response};
use feather_runtime::runtime::engine::Engine;
use std::borrow::Cow;
use std::collections::HashMap;
use std::{fmt::Display, net::ToSocketAddrs};

/// A route in the application.  
#[repr(C)]
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
    error_handler: Option<ErrorHandler>,
}

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

impl App {
    /// Create a new instance of the application
    /// Also initializes the Logger if the `log` feature is enabled.
    /// # Example
    /// ```rust,ignore
    /// use feather::App;
    /// fn main() {
    ///     let mut app = App::new();
    ///     // do stuff with app
    /// }
    /// ```
    #[must_use = "Does nothing if you don't use the `listen` method"]
    pub fn new() -> Self {
        #[cfg(feature = "log")]
        #[cfg(debug_assertions)]
        {
            use std::sync::Once;
            static INIT_LOGGER: Once = Once::new();
            INIT_LOGGER.call_once(|| {
                simple_logger::SimpleLogger::new()
                    .with_module_level("may", log::LevelFilter::Off)
                    .with_module_level("feather_runtime", log::LevelFilter::Off)
                    .init()
                    .expect("Failed to initialize logger.");
            });
        }
        Self {
            routes: Vec::new(),
            middleware: Vec::new(),
            context: AppContext::new(),
            error_handler: None,
        }
    }
    /// Create a new instance of the application without initializing the logger.
    /// This is useful if you want to manage logging yourself or use a different logging solution.
    pub fn without_logger() -> Self {
        Self {
            routes: Vec::new(),
            middleware: Vec::new(),
            context: AppContext::new(),
            error_handler: None,
        }
    }
    /// Returns a Handle to the [AppContext] inside the App
    /// [AppContext] is Used for App wide state managment
    #[inline]
    pub fn context(&mut self) -> &mut AppContext {
        &mut self.context
    }
    /// Set up the Error Handling Solution for that [App].  
    /// If there are no Error Handler present by default,  
    /// framework will Catch the error and print it to the `stderr` and return a `500` Status code response back to the client
    #[inline]
    pub fn set_error_handler(&mut self, handler: ErrorHandler) {
        self.error_handler = Some(handler)
    }

    /// Add a route to the application.  
    /// Every Route Returns A MiddlewareResult to control the flow of your application.
    #[inline]
    pub fn route<M: Middleware + 'static>(&mut self, method: Method, path: impl Into<Cow<'static, str>>, middleware: M) {
        self.routes.push(Route {
            method,
            path: path.into(),
            middleware: Box::new(middleware),
        });
    }

    /// Add a global middleware to the application that will be applied to all routes.
    #[inline]
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

    fn run_middleware(mut request: &mut Request, routes: &[Route], global_middleware: &[Box<dyn Middleware>], mut context: &mut AppContext, error_handler: &Option<ErrorHandler>) -> Response {
        let mut response = Response::default();
        // Run global middleware

        for middleware in global_middleware {
            match middleware.handle(&mut request, &mut response, &mut context) {
                Ok(_) => {}
                Err(e) => {
                    if let Some(handler) = &error_handler {
                        handler(e, &request, &mut response)
                    } else {
                        eprintln!("Unhandled Error caught in middlewares: {}", e);
                        response.set_status(500).send_text("Internal Server Error!");
                        return response;
                    }
                }
            }
        }

        // Run route-specific middleware with dynamic route matching
        let mut found = false;
        for route in routes.iter().filter(|r| r.method == request.method) {
            if let Some(params) = Self::match_route(&route.path, &request.path()) {
                request.set_params(params);
                match route.middleware.handle(request, &mut response, &mut context) {
                    Ok(_) => {}
                    Err(e) => {
                        if let Some(handler) = &error_handler {
                            handler(e, &request, &mut response)
                        } else {
                            eprintln!("Unhandled Error caught in Route Middlewares : {}", e);
                            response.set_status(500).send_text("Internal Server Error");
                        }
                    }
                }
                found = true;
                break;
            }
        }
        if !found {
            response.set_status(404).send_text("404 Not Found");
        }

        response
    }

    fn match_route<'r>(pattern: &'r str, path: &'r str) -> Option<HashMap<String, String>> {
        let mut params = HashMap::new();
        let pattern_parts: Vec<&str> = pattern.trim_matches('/').split('/').collect();
        let path_parts: Vec<&str> = path.trim_matches('/').split('/').collect();

        if pattern_parts.len() != path_parts.len() {
            return None;
        }

        for (pat, val) in pattern_parts.iter().zip(path_parts.iter()) {
            if pat.starts_with(':') {
                params.insert(pat[1..].to_string(), val.to_string());
            } else if pat != val {
                return None;
            }
        }

        Some(params)
    }

    /// Start the application and listen for incoming requests on the given address.
    /// Blocks the current thread until the server is stopped.
    ///
    /// # Panics
    ///
    /// Panics if the server fails to start
    #[inline]
    pub fn listen(&mut self, address: impl ToSocketAddrs + Display) {
        
        println!("Feather listening on : http://{address}",);
        let rt = Engine::new(address);
        let routes = &self.routes;
        let middleware = &self.middleware;
        let mut ctx = &mut self.context;
        let error_handle = &self.error_handler;
        rt.start();
        rt.for_each(move |mut req| {
            let response = Self::run_middleware(&mut req, &routes, &middleware, &mut ctx, error_handle);
            return response;
        })
        .unwrap();
    }
}
