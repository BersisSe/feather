// Import dependencies from Feather
use feather::{App, AppContext, Request, Response};
// Import the Middleware trait and some common middleware primitives
use feather::middleware::builtins;
use feather::middleware::{Middleware, MiddlewareResult};
// Implementors of the Middleware trait are middleware that can be used in a Feather app.
struct Custom;

// The Middleware trait defines a single method `handle`,
// which can mutate the request and response objects, then return a `MiddlewareResult`.
impl Middleware for Custom {
    fn handle(
        &self,
        request: &mut Request,
        _response: &mut Response,
        _ctx: &mut AppContext,
    ) -> MiddlewareResult {
        // Do stuff here
        println!("Now running some custom middleware (struct Custom)!");
        println!("And there's a request with path: {:?}", request.uri);
        // and then continue to the next middleware in the chain
        MiddlewareResult::Next
    }
}

fn main() {
    // Create a new instance of App
    let mut app = App::new();

    // Use the builtin Logger middleware for all routes
    app.use_middleware(builtins::Logger);

    // Use the Custom middleware for all routes
    app.use_middleware(Custom);

    // Use another middleware defined by a function for all routes
    app.use_middleware(
        |_request: &mut Request, _response: &mut Response, _ctx: &mut AppContext| {
            println!("Now running some custom middleware (closure)!");
            MiddlewareResult::Next
        },
    );

    // Define a route
    app.get(
        "/",
        |_request: &mut Request, response: &mut Response, _ctx: &mut AppContext| {
            response.send_text("Hello, world!");
            MiddlewareResult::Next
        },
    );

    // Listen on port 3000
    app.listen("127.0.0.1:3000");
}
