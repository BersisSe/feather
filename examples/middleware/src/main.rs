use feather::{App, AppContext, Outcome, Request, Response, chain, middlewares::builtins, next};
mod middleware;
use middleware::MyMiddleware;
fn main() {
    let mut app = App::new();

    app.use_middleware(builtins::Logger); // We can easily use middlewares using this syntax
    // We can also put Closures as a middleware parameter. that what makes Feather "Middleware-First"
    app.use_middleware(|_req: &mut feather::Request, _res: &mut feather::Response, _ctx: &mut feather::AppContext| {
        println!("Ow a Request: I am a Closure Middleware BTW");
        next!()
    });
    app.use_middleware(MyMiddleware("Secret Codee".to_string()));

    app.get("/", |_req: &mut Request, res: &mut Response, _ctx: &mut AppContext| {
        res.send_text("Hellooo Feather Middleware Example");
        res.set_status(200);
        next!()
    });
    // You can also chain middlewares using the `chain!` macro
    // the first given middleware will always run first!
    // You can also chain more than 2 middlewares
    app.get("/chain", chain!(first, second));

    app.listen("127.0.0.1:5050");
}

fn first(_req: &mut Request, res: &mut Response, _ctx: &mut AppContext) -> Outcome {
    println!("First Middleware Ran");
    res.set_status(201);
    next!()
}
fn second(_req: &mut Request, res: &mut Response, _ctx: &mut AppContext) -> Outcome {
    println!("Second Ran");
    res.send_text("Yep Chained middlewares");
    next!()
}
