use std::time::Duration;

use feather::{App, Outcome, end, info, middleware_fn, next, prelude::async_middleware};

fn main() {
    let mut app = App::new();
    app.workers(20);
    app.get("/", first);
    app.get("/sync", sec);
    app.listen("127.0.0.1:5050");
    
}
/// Create async middleware using futures-timer (executor-agnostic)
#[async_middleware]
async fn first() -> Outcome{
    info!("First Middleware Ran");
    // Use futures_timer which works with any executor
    futures_timer::Delay::new(Duration::from_secs(5)).await;
    info!("Very Intensive Thing done!");
    res.send_html("<h1>Done!</h1>");
    res.add_header("Connection", "close").ok();
    res.set_status(201);
    next!()
}

#[async_middleware]
async fn sec() -> Outcome{
    use std::time::SystemTime;
    use std::thread;
    
    let time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    
    let before = SystemTime::now();
    println!("[{}] Request started (thread: {:?})", time, thread::current().id());
    
    futures_timer::Delay::new(Duration::from_secs(5)).await;
    
    let elapsed = before.elapsed().unwrap();
    let after = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    
    println!("[{}] Request finished. Elapsed: {}ms", after, elapsed.as_millis());
    
    res.send_text("done");
    res.add_header("Connection", "close").ok();
    next!()
}