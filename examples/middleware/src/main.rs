use feather::{chain, middleware, middlewares::builtins, next, App, middleware_fn,info};
mod middleware;
use middleware::MyMiddleware;
fn main() {
    let mut app = App::new();

    app.use_middleware(builtins::Logger); // We can easily use middlewares using this syntax
    // We can also put Closures as a middleware parameter. that what makes Feather "Middleware-First"
    app.use_middleware(middleware!(|_req, _res, _ctx| {
        info!("Custom global middleware!");
        next!()
    }));
    app.use_middleware(MyMiddleware("Secret Code".to_string()));

    app.get(
        "/",
        middleware!(|_req, res, _ctx| {
            res.send_text("Hellooo Feather Middleware Example");
            res.set_status(200);
            next!()
        }),
    );
    // You can also chain middlewares using the `chain!` macro
    // the first given middleware will always run first!
    // You can also chain more than 2 middlewares
    app.get("/chain", chain!(first, second));

    app.listen("127.0.0.1:5050");
}

#[middleware_fn]
fn first() -> Outcome {
    info!("First Middleware Ran");
    res.set_status(201);
    next!()
}
#[middleware_fn]
fn second() -> Outcome {
    info!("Second Ran");
    res.send_text("Yep Chained middlewares");
    next!()
}
