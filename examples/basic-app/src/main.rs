// Import dependencies from Feather
use feather::middleware;
use feather::middlewares::builtins;
use feather::{App, next};

// Main function
fn main() {
    // Create a new instance of App
    let mut app = App::new();
    // Define a route for the root path
    app.get(
        "/",
        middleware!(|_request, response, _ctx| {
            response.send_text("Hello, world!");
            next!()
        }),
    );

    // Use the Logger middleware for all routes
    app.use_middleware(builtins::Logger);
    // Listen on port 5050
    app.listen("127.0.0.1:5050");
}
