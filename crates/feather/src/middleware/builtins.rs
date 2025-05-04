use crate::internals::AppContext;

use super::common::{Middleware, MiddlewareResult};
use feather_runtime::http::{Request, Response};
use std::{
    fs::{self, File},
    io::Read,
    path::Path,
};

/// Log incoming requests and transparently pass them to the next middleware.
pub struct Logger;

impl Middleware for Logger {
    fn handle(
        &self,
        request: &mut Request,
        _: &mut Response,
        _: &mut AppContext,
    ) -> MiddlewareResult {
        println!("Request: {request}");
        MiddlewareResult::Next
    }
}

#[derive(Default)]
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
    fn handle(
        &self,
        _: &mut Request,
        response: &mut Response,
        _: &mut AppContext,
    ) -> MiddlewareResult {
        response.add_header(
            "Access-Control-Allow-Origin",
            self.0.as_deref().unwrap_or("*"),
        );
        MiddlewareResult::Next
    }
}

/// Serve static files from the given path.
pub struct ServeStatic(String);

impl ServeStatic {
    #[must_use = "Put this in a `App.use_middleware()` Method"]
    pub const fn new(directory: String) -> Self {
        Self(directory)
    }
}

impl Middleware for ServeStatic {
    fn handle(
        &self,
        request: &mut Request,
        response: &mut Response,
        _: &mut AppContext,
    ) -> MiddlewareResult {
        let wanted_path = request.uri.to_string();
        let dir = fs::read_dir(Path::new(self.0.as_str()))
            .expect(format!("Error While Reading the {}", self.0).as_str());
        dir.for_each(|entry| {
            let entry = entry.unwrap();
            let enter_path = entry.path();
            let mut file = File::open(entry.path()).unwrap();
            let ext = enter_path.extension().unwrap();
            match ext.to_str().unwrap() {
                "png" => {
                    if wanted_path.contains(entry.file_name().into_string().unwrap().as_str()) {
                        response.add_header("Content-Type", "image/png");
                        let mut buf = Vec::with_capacity(4096);
                        file.read(&mut buf).unwrap();
                        response.send_bytes(buf);
                    }
                }
                "jpg" => {
                    if wanted_path.contains(entry.file_name().into_string().unwrap().as_str()) {
                        response.add_header("Content-Type", "image/jpg");
                        let mut buf = Vec::with_capacity(1024);
                        file.read_to_end(&mut buf).unwrap();
                        response.send_bytes(buf);
                    }
                }
                "jpeg" => {
                    if wanted_path.contains(entry.file_name().into_string().unwrap().as_str()) {
                        response.add_header("Content-Type", "image/jpeg");
                        let mut buf = Vec::with_capacity(1024);
                        file.read_to_end(&mut buf).unwrap();
                        response.send_bytes(buf);
                    }
                }
                "html" => {
                    if wanted_path.contains(entry.file_name().into_string().unwrap().as_str()) {
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
