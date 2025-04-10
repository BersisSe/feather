use feather::middleware::{MiddlewareResult, ServeStatic};
use feather::{App, Request, Response};

fn main() {
    // Create a new instance of App
    let mut app = App::new();
    // Define a route for the root path
    app.get("/", |_req: &mut Request, res: &mut Response| {
        res.send_text("Hello, world!");
        MiddlewareResult::Next
    });
    // Use the ServeStatic middleware to serve static files from the "public" directory

    app.use_middleware(ServeStatic::new("./public".to_string()));// You can change the path to your static files here

    //Lets Listen on port 8080
    app.listen("127.0.0.1:8080");
}
