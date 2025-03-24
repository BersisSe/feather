use crate::middleware::Middleware;
use crate::response::Response;
use std::{
    fmt::Display,
    net::ToSocketAddrs,
    sync::{Arc, RwLock},
};
use threadpool::ThreadPool;
use tiny_http::{Header, Method, Server};

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
            pub fn $name<M: Middleware + 'static>(&mut self, path: impl Into<String>, middleware: M)
            {
                self.route(Method::$method, path.into(), middleware);
            }
        )+
    }
}

impl App {
    #[must_use]
    pub fn new(config: AppConfig) -> Self {
        Self {
            config,
            routes: Arc::new(RwLock::new(Vec::new())),
            middleware: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Add a route to the application.
    ///
    /// # Panics
    ///
    /// Panics if the internal [`RwLock`] protecting the routes is poisoned.
    pub fn route<M: Middleware + 'static>(&mut self, method: Method, path: String, middleware: M) {
        self.routes.write().unwrap().push(Route {
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
        self.middleware.write().unwrap().push(Box::new(middleware));
    }

    route_methods!(
        Get get
        Post post
        Put put
        Delete delete
        Patch patch
        Head head
        Options options
    );

    /// Start the application and listen for incoming requests on the given address.
    /// Blocks the current thread until the server is stopped.
    ///
    /// # Panics
    ///
    /// Panics if the server fails to start or if the internal [`RwLock`]s protecting the routes
    /// or middleware are poisoned.
    pub fn listen(&self, address: impl ToSocketAddrs + Display) {
        let server = Arc::new(Server::http(&address).expect("Failed to start server"));
        eprintln!("Feather listening on http://{address}");
        let pool = ThreadPool::new(self.config.threads);

        for mut request in server.incoming_requests() {
            let routes = Arc::clone(&self.routes);
            let middleware = Arc::clone(&self.middleware);
            pool.execute(move || {
                let mut response = Response::default();
                for middleware in &*middleware.read().unwrap() {
                    middleware.handle(&mut request, &mut response);
                }
                for Route {
                    method,
                    path,
                    middleware,
                } in &*routes.read().unwrap()
                {
                    if method != request.method() || *path != request.url() {
                        continue;
                    }
                    middleware.handle(&mut request, &mut response);
                }
                let result = if let Some(file) = response.file {
                    request.respond(response.headers.into_iter().fold(
                        tiny_http::Response::from_file(file).with_status_code(response.status_code),
                        |response, (key, value)| {
                            response.with_header(
                                Header::from_bytes(key.as_bytes(), value.as_bytes()).unwrap(),
                            )
                        },
                    ))
                } else if let Some(body) = response.body {
                    request.respond(
                        response.headers.into_iter().fold(
                            tiny_http::Response::from_string(body)
                                .with_status_code(response.status_code),
                            |response, (key, value)| {
                                response.with_header(
                                    Header::from_bytes(key.as_bytes(), value.as_bytes()).unwrap(),
                                )
                            },
                        ),
                    )
                } else {
                    return;
                };
                if let Err(e) = result {
                    eprintln!("Response failed to send: {e}");
                }
            });

            pool.join();
        }
    }
}
