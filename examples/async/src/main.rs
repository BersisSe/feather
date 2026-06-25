//! This example demonstrates how to use the async compat layer to interact with async tooling
//! Still async compat layer is really new and very experimental so expect bugs, and we do not recommend using it in a production setting


use std::time::Duration;
use feather::{App, info, next, async_compat::async_middleware};

fn main() {
    let mut app = App::new();
    app.workers(20);
    app.get("/", first);
    app.listen("127.0.0.1:5050");
}

/// Create async middleware and simulate a async operation using a delay
#[async_middleware]
async fn first() -> Outcome{
    info!("First Middleware Ran");
    futures_timer::Delay::new(Duration::from_secs(5)).await;
    info!("Very Intensive Thing done!");
    res.send_html("<h1>Done!</h1>");
    res.add_header("Connection", "close").ok();
    res.set_status(201);
    next!()
}