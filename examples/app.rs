// Import dependencies from Feather
use feather::HttpResponse;
use feather::middleware::{Logger, MiddlewareResult, Cors};
use feather::{App, AppConfig};
// Main function - no async here!
fn main() {
    // Create instance of AppConfig with 4 threads
    let config = AppConfig { threads: 32 };

    // Create a new instance of App
    let mut app = App::new(config);
    app.use_middleware(Cors::default());
    // Define a route for the root path
    app.get("/", |_request: &mut _, response: &mut _| {
        *response = HttpResponse::ok("Hello from Feather!");
        MiddlewareResult::Next
    });
    // Use the Logger middleware for all routes
    app.use_middleware(Logger);
    // Listen on port 3000
    app.listen("127.0.0.1:3000");
}
