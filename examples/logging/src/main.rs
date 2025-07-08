use feather::{info, next, App, AppContext, Request, Response};

// Example: Logging requests in Feather
// Demonstrates how to log incoming requests using the info! macro.

fn main() {     
    let mut app = App::new();
    // Log Example
    app.get("/", |req: &mut Request, res: &mut Response, _ctx: &mut AppContext| {
        info!("Received a request: {}", req);
        res.send_text("Hello, World!");
        next!()
    });

    app.listen("127.0.0.1:5050");
}
