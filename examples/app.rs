// Import dependencies from Feather
use feather::{App, MiddlewareResult};
use feather::middleware::Logger;
use feather::{Request, Response};

// Main function - no async here!
fn main() {
    // Create a new instance of App
    let mut app = App::new();
    
    // Define a route for the root path
    app.get("/", |_request: &mut Request, response: &mut Response| {
        response.send_text("Hello, world!");
        MiddlewareResult::Next
    });
    // Use the Logger middleware for all routes
    app.use_middleware(Logger);
    // Listen on port 3000
    app.listen("127.0.0.1:3000");
}
