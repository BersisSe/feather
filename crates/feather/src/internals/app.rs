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
#[repr(C)]
pub struct Route {
    pub method: Method,
    pub path: Cow<'static, str>,
    pub middleware: Box<dyn Middleware>,
}

/// A Feather application.  

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
    /// Returns a Handle to the [AppContext] inside the App
    /// [AppContext] is Used for App wide state managment
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
    /// Default is 65536 bytes (64KB).
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

    /// Start the application and listen for incoming requests on the given address.
    /// Blocks the current thread until the server is stopped.
    ///
    /// # Panics
    ///
    /// Panics if the server fails to start
    #[inline]
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
