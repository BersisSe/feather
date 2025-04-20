use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use dyn_clone::DynClone;
use feather_runtime::http::HttpRequest as Request;
use feather_runtime::http::HttpResponse as Response;
/// Common trait for all middleware types. Implemented automatically for functions fitting
/// the `(request, response) -> result` signature.

pub trait Middleware: Send + Sync + DynClone {
    /// Handle an incoming request by transforming it into a response.
    fn handle(&self, request: &mut Request, response: &mut Response) -> MiddlewareResult;
}
dyn_clone::clone_trait_object!(Middleware);

/// MiddlewareResult is used to control the flow of middleware execution.
/// 
/// It can be used to skip all subsequent middleware and continue to the next route.
pub enum MiddlewareResult {
    /// Continue to the next middleware.
    Next,
    /// Skip all subsequent middleware and continue to the next route.
    NextRoute,
}

#[derive(Clone)]
/// Log incoming requests and transparently pass them to the next middleware.
pub struct Logger;

impl Middleware for Logger {
    fn handle(&self, request: &mut Request, _: &mut Response) -> MiddlewareResult {
        println!("Request: {request}");
        MiddlewareResult::Next
    }
}

#[derive(Clone, Default)]
/// Add [CORS] headers to the response.
///
/// [CORS]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Guides/CORS
pub struct Cors(Option<String>);

impl Cors {
    #[must_use]
    pub const fn new(origin: String) -> Self {
        Self(Some(origin))
    }
}

impl Middleware for Cors {
    fn handle(&self, _: &mut Request, response: &mut Response) -> MiddlewareResult {
        response.add_header(
            "Access-Control-Allow-Origin",
            self.0.as_deref().unwrap_or("*"),
        );
        MiddlewareResult::Next
    }
}

#[derive(Clone)]
/// Serve static files from the given path.
pub struct ServeStatic(String);

impl ServeStatic {
    #[must_use= "Put this in a `App.use_middleware()` Method"]
    pub const fn new(directory: String) -> Self {
        Self(directory)
    }
}

impl Middleware for ServeStatic {
    fn handle(&self, request: &mut Request, response: &mut Response) -> MiddlewareResult {
        let wanted_path = request.uri.to_string();
        let dir = fs::read_dir(Path::new(self.0.as_str())).expect(format!("Error While Reading the {}",self.0).as_str());
        dir.for_each(|entry|{
            let entry = entry.unwrap();
            let enter_path = entry.path();
            let mut file = File::open(entry.path()).unwrap();
            let ext = enter_path.extension().unwrap();
            match ext.to_str().unwrap() {
                "png" => {
                    if wanted_path.contains(entry.file_name().into_string().unwrap().as_str()){
                        response.add_header("Content-Type", "image/png");
                        let mut buf = Vec::with_capacity(4096);
                        file.read(&mut buf).unwrap();
                        response.send_bytes(buf);
                    }
                }
                "jpg" =>{
                    if wanted_path.contains(entry.file_name().into_string().unwrap().as_str()){
                        response.add_header("Content-Type", "image/jpg");
                        let mut buf = Vec::with_capacity(1024);
                        file.read_to_end(&mut buf).unwrap();
                        response.send_bytes(buf);
                    }
                }
                "jpeg" =>{
                    if wanted_path.contains(entry.file_name().into_string().unwrap().as_str()){
                        response.add_header("Content-Type", "image/jpeg");
                        let mut buf = Vec::with_capacity(1024);
                        file.read_to_end(&mut buf).unwrap();
                        response.send_bytes(buf);
                    }
                }
                "html" =>{
                    if wanted_path.contains(entry.file_name().into_string().unwrap().as_str()){
                        response.add_header("Content-Type", "text/html");
                        let mut buf = Vec::with_capacity(1024);
                        file.read_to_end(&mut buf).unwrap();
                        response.send_bytes(buf);
                    }
                }
                
                
                _ => unreachable!(),
            }
            
        });
        MiddlewareResult::Next
    }
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
