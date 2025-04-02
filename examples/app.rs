// Import dependencies from Feather
use feather::Response;
use feather::middleware::{Logger, MiddlewareResult};
use feather::{App, AppConfig};
// Main function - no async here!
fn main() {
    // Create instance of AppConfig with 4 threads
    let config = AppConfig { threads: 4 };

    // Create a new instance of App
    let mut app = App::new(config);

    // Define a route for the root path
    app.get("/", |_request: &mut _, response: &mut _| {
        *response = Response::ok("Hello from Feather!");
        MiddlewareResult::Next
    });
    // Use the Logger middleware for all routes
    app.use_middleware(Logger);
    // Listen on port 3000
    app.listen("127.0.0.1:3000");
}
