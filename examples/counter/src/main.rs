use feather::{next, App, AppContext, Request, Response};
// Create a couter struct to hold the state
#[derive(Debug)]
struct Counter {
    pub count: i32,
}

fn main() {
    let mut app = App::new();
    let counter = Counter { count: 0 };
    // Put the counter in the app context
    app.context().set_state(counter);

    app.get(
        "/",
        move |_req: &mut Request, res: &mut Response, ctx: &mut AppContext| {
            let counter: &mut Counter = ctx.get_mut_state::<Counter>().unwrap();
            counter.count += 1;// Increment the counter for every request
            // Send the current count as a response
            res.send_text(format!("Counted! {}", counter.count));
            next!()
        },
    );
    // Lastly add a route to get the current count
    app.get(
        "/count",
        move |_req: &mut Request, res: &mut Response, ctx: &mut AppContext| {
            let counter = ctx.get_state::<Counter>().unwrap();
            res.send_text(counter.count.to_string());
            next!()
        },
    );

    app.listen("127.0.0.1:5050");
}
