use std::time::Duration;

use feather::{App, info, middleware_fn, next};
use feather::async_compat::async_middleware;

fn main() {
    let mut app = App::new();
    app.get("/", first);
    app.listen("127.0.0.1:5050");
    
}

#[async_middleware]
async fn first() -> Outcome {
    info!("First Middleware Ran");
    smol::Timer::after(Duration::from_secs(5)).await;
    info!("Very Intensive Thing done!");
    res.send_html("<h1></h1>");
    res.set_status(201);
    next!()
}

