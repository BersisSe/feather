use feather::{App, middleware, next};

// Example: Dynamic route parameters with Feather
// This demonstrates how to define and extract a dynamic path parameter (e.g., /users/:id).

fn main() {
    let mut app = App::new();
    // Define a Route With a Dynamic Path
    app.get(
        "/users/:id",
        middleware!(|req, res, _ctx| {
            let id = req.param("id");
            res.send_text(format!("Welcome User: {}", id.unwrap()));
            next!()
        }),
    );

    // Lets Listen on port 5050
    app.listen("127.0.0.1:5050");
}
