use feather::middlewares::builtins::ServeStatic;
use feather::{App,middleware,next};
// To Use this example you need to have a 'public' directory with some static files in it
// in the same directory as this file.
// This example shows how to use the ServeStatic middleware to serve static files from a directory.
fn main() {
    // Create a new instance of App
    let mut app = App::new();
    // Define a route for the root path
    app.get(
        "/",
        middleware!(|_req, res, _ctx| {
            res.send_text("Hello, world!");
            next!()
        }),
    );
    // Use the ServeStatic middleware to serve static files from the "public" directory
    app.use_middleware(ServeStatic::new("./public".to_string())); // You can change the path to your static files here

    //Lets Listen on port 5050
    app.listen("127.0.0.1:5050");
}
