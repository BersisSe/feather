// Import dependencies from Feather
use feather::middleware::builtins;
use feather::{App, AppContext, MiddlewareResult};
use feather::{Request, Response};

// Main function - no async here!
fn main() {
    // Create a new instance of App
    let mut app = App::new();

    // Define a route for the root path
    app.get(
        "/",
        |_request: &mut Request, response: &mut Response, _ctx: &mut AppContext| {
            response.send_text("Hello, world!");
            MiddlewareResult::Next
        },
    );
    // Use the Logger middleware for all routes
    app.use_middleware(builtins::Logger);
    // Listen on port 3000
    app.listen("127.0.0.1:3000");
}
