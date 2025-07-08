use feather::{App, middleware, next};
// Create a couter struct to hold the state
#[derive(Debug)]
struct Counter {
    pub count: i32,
}
impl Counter {
    // Increment the counter
    pub fn increment(&mut self) {
        self.count += 1;
    }
    // Decrement the counter
    pub fn decrement(&mut self) {
        self.count -= 1;
    }
}

fn main() {
    let mut app = App::new();
    let counter = Counter {
        count: 0,
    };
    // Put the counter in the app context
    app.context().set_state(counter);

    app.get(
        "/",
        middleware!(|_req, res, ctx| {
            let counter = ctx.get_state::<Counter>().unwrap();
            res.send_text(format!("Counter value: {}", counter.count));
            next!()
        }),
    );
    // Lastly add a route to get the current count
    app.get(
        "/count",
        middleware!(|_req, res, ctx| {
            let counter = ctx.get_state::<Counter>().unwrap();
            res.send_text(counter.count.to_string());
            next!()
        }),
    );

    // Route to increment the counter
    app.post(
        "/increment",
        middleware!(|_req, res, ctx| {
            let counter = ctx.get_state_mut::<Counter>().unwrap();
            counter.increment();
            res.send_text(format!("Counter incremented: {}", counter.count));
            next!()
        }),
    );

    // Route to decrement the counter
    app.post(
        "/decrement",
        middleware!(|_req, res, ctx| {
            let counter = ctx.get_state_mut::<Counter>().unwrap();
            counter.decrement();
            res.send_text(format!("Counter decremented: {}", counter.count));
            next!()
        }),
    );

    app.listen("127.0.0.1:5050");
}
