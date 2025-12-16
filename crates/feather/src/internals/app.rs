use super::AppContext;
use super::error_stack::ErrorHandler;
use crate::internals::service::AppService;
use crate::middlewares::Middleware;
pub use feather_runtime::Method;
pub use feather_runtime::runtime::server::ServerConfig;
use feather_runtime::runtime::server::Server;

use std::borrow::Cow;

use std::{fmt::Display, net::ToSocketAddrs};

/// A route in the application.
///
/// Routes map HTTP methods and paths to middleware handlers.
#[repr(C)]
pub struct Route {
    pub method: Method,
    pub path: Cow<'static, str>,
    pub middleware: Box<dyn Middleware>,
}

/// A Feather application.
///
/// The main entry point for building web applications. Create an instance,
/// add routes and middleware, then call `listen()` to start the server.
///
/// # Example
///
/// ```rust,ignore
/// use feather::{App, middleware, next};
///
/// let mut app = App::new();
///
/// app.get("/", middleware!(|_req, res, _ctx| {
///     res.send_text("Hello, Feather!");
///     next!()
/// }));
///
/// app.listen("127.0.0.1:5050");
/// ```
pub struct App {
    routes: Vec<Route>,
    middleware: Vec<Box<dyn Middleware>>,
    context: AppContext,
    error_handler: Option<ErrorHandler>,
    server_config: ServerConfig,
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
                use tracing_subscriber::filter::filter_fn;
                use tracing_subscriber::{Layer, prelude::*};

                let layer = tracing_subscriber::fmt::layer()
                    .with_target(false)
                    .pretty()
                    .compact()
                    .with_target(false)
                    .with_thread_ids(false)
                    .with_level(true)
                    .with_filter(filter_fn(|meta| {
                        // Ignore logs from feather_runtime and may crates
                        !meta.target().starts_with("feather_runtime") && !meta.target().starts_with("may")
                    }))
                    .boxed();

                tracing_subscriber::registry().with(layer).init();
            });
        }
        Self {
            routes: Vec::new(),
            middleware: Vec::new(),
            context: AppContext::new(),
            error_handler: None,
            server_config: ServerConfig::default(),
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
            server_config: ServerConfig::default(),
        }
    }

    /// Create a new instance of the application with a custom server configuration.
    /// # Example
    /// ```rust,ignore
    /// use feather::{App, ServerConfig};
    /// 
    /// let config = ServerConfig {
    ///     max_body_size: 10 * 1024 * 1024,  // 10MB
    ///     read_timeout_secs: 60,             // 60 seconds
    ///     workers: 4,                        // 4 worker threads
    ///     stack_size: 128 * 1024,            // 128KB
    /// };
    /// 
    /// let mut app = App::with_config(config);
    /// ```
    pub fn with_config(config: ServerConfig) -> Self {
        #[cfg(feature = "log")]
        #[cfg(debug_assertions)]
        {
            use std::sync::Once;
            static INIT_LOGGER: Once = Once::new();

            INIT_LOGGER.call_once(|| {
                use tracing_subscriber::filter::filter_fn;
                use tracing_subscriber::{Layer, prelude::*};

                let layer = tracing_subscriber::fmt::layer()
                    .with_target(false)
                    .pretty()
                    .compact()
                    .with_target(false)
                    .with_thread_ids(false)
                    .with_level(true)
                    .with_filter(filter_fn(|meta| {
                        // Ignore logs from feather_runtime and may crates
                        !meta.target().starts_with("feather_runtime") && !meta.target().starts_with("may")
                    }))
                    .boxed();

                tracing_subscriber::registry().with(layer).init();
            });
        }
        Self {
            routes: Vec::new(),
            middleware: Vec::new(),
            context: AppContext::new(),
            error_handler: None,
            server_config: config,
        }
    }
    /// Returns a mutable reference to the [AppContext].
    ///
    /// The context is used for application-wide state management. Use it to store
    /// and retrieve data that needs to be shared across requests.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use feather::State;
    ///
    /// struct Config {
    ///     database_url: String,
    /// }
    ///
    /// let mut app = App::new();
    /// app.context().set_state(State::new(Config {
    ///     database_url: "postgresql://localhost/db".to_string(),
    /// }));
    /// ```
    pub fn context(&mut self) -> &mut AppContext {
        &mut self.context
    }
    /// Set up custom error handling for the application.
    ///
    /// By default, Feather catches errors and returns a 500 response. Use this to
    /// customize error handling behavior.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let error_handler = |err, _req, res| {
    ///     eprintln!("Error: {}", err);
    ///     // Customize error response
    /// };
    ///
    /// app.set_error_handler(Box::new(error_handler));
    /// ```
    #[inline]
    pub fn set_error_handler(&mut self, handler: ErrorHandler) {
        self.error_handler = Some(handler)
    }

    /// Set the maximum request body size in bytes.
    /// Default is 8192 bytes (8KB).
    /// # Example
    /// ```rust,ignore
    /// app.max_body(10 * 1024 * 1024); // 10MB
    /// ```
    #[inline]
    pub fn max_body(&mut self, size: usize) -> &mut Self {
        self.server_config.max_body_size = size;
        self
    }

    /// Set the read timeout in seconds for client connections.
    /// Default is 30 seconds.
    /// # Example
    /// ```rust,ignore
    /// app.read_timeout(60); // 60 seconds
    /// ```
    #[inline]
    pub fn read_timeout(&mut self, seconds: u64) -> &mut Self {
        self.server_config.read_timeout_secs = seconds;
        self
    }

    /// Set the number of worker threads for handling connections.
    /// Default is the number of CPU cores.
    /// # Example
    /// ```rust,ignore
    /// app.workers(4); // 4 worker threads
    /// ```
    #[inline]
    pub fn workers(&mut self, count: usize) -> &mut Self {
        self.server_config.workers = count;
        self
    }

    /// Set the stack size per coroutine in bytes.  
    /// Default is 65536 bytes (64KB).<br>
    /// **Using Stack Size lower than 32KB can create Stack Overflow issues with the logger.**  
    /// # Example
    /// ```rust,ignore
    /// app.stack_size(128 * 1024); // 128KB
    /// ```
    #[inline]
    pub fn stack_size(&mut self, size: usize) -> &mut Self {
        self.server_config.stack_size = size;
        self
    }

    /// Add a route to the application.
    ///
    /// This is the generic method for adding routes. For convenience, use the
    /// HTTP method-specific methods like `get()`, `post()`, etc.
    ///
    /// # Arguments
    ///
    /// * `method` - The HTTP method (GET, POST, etc.)
    /// * `path` - The route path (e.g., "/users/:id")
    /// * `middleware` - The middleware handler for this route
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use feather::{App, Method};
    ///
    /// let mut app = App::new();
    /// app.route(Method::GET, "/", middleware!(|_req, res, _ctx| {
    ///     res.send_text("Hello");
    ///     next!()
    /// }));
    /// ```
    #[inline]
    pub fn route<M: Middleware + 'static>(&mut self, method: Method, path: impl Into<Cow<'static, str>>, middleware: M) {
        self.routes.push(Route {
            method,
            path: path.into(),
            middleware: Box::new(middleware),
        });
    }

    /// Add a global middleware to the application that will be applied to all routes.
    ///
    /// Global middleware runs on every request before any route-specific middleware.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// app.use_middleware(middleware!(|req, res, _ctx| {
    ///     println!("Request to: {}", req.uri);
    ///     next!()
    /// }));
    /// ```
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

    /// Start the application and listen for incoming requests.
    ///
    /// This method blocks the current thread and starts accepting connections on
    /// the specified address. The server will continue running until the process exits.
    ///
    /// # Arguments
    ///
    /// * `address` - The address to bind to (e.g., "127.0.0.1:5050")
    ///
    /// # Panics
    ///
    /// Panics if the server fails to bind to the specified address.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// app.listen("127.0.0.1:5050");
    /// ```
    pub fn listen(self, address: impl ToSocketAddrs + Display) {
        let svc = AppService {
            routes: self.routes,
            middleware: self.middleware,
            context: self.context,
            error_handler: self.error_handler,
        };
        println!("Feather listening on : http://{address}",);
        Server::with_config(svc, self.server_config).run(address).expect("Failed to start server");
    }
}
