use feather::{App, Finalizer, middleware, middleware_fn, next};
mod api;

#[middleware_fn]
fn global_logger() {
    println!("[Global] {} {}", req.method, req.uri);
    next!()
}

fn main() {
    let mut app = App::new();

    // 1. Add global middleware
    app.use_middleware(global_logger);

    // 2. Add a basic root route
    app.get("/", middleware!(|_req, res, _ctx| { res.finish_text("Welcome to the Home Page") }));

    // 3. Mount the sub-router
    // This will result in a route: GET /api/v1/data
    app.mount("/api/v1", api::api_router());

    app.listen("127.0.0.1:5050");
}
