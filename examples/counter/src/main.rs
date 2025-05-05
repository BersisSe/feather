use feather::{App, AppContext, MiddlewareResult, Request, Response};

struct Counter {
    pub count: i32,
}

fn main() {
    let mut app = App::new();
    let counter = Counter { count: 0 };
    app.context().set_state(counter);

    app.get(
        "/",
        move |_req: &mut Request, res: &mut Response, ctx: &mut AppContext| {
            let counter: &mut Counter = ctx.get_mut_state::<Counter>().unwrap();
            counter.count += 1;
            res.send_text(format!("Counted! {}", counter.count));
            MiddlewareResult::Next
        },
    );

    app.get(
        "/count",
        move |_req: &mut Request, res: &mut Response, ctx: &mut AppContext| {
            let counter = ctx.get_state::<Counter>().unwrap();
            res.send_text(counter.count.to_string());
            MiddlewareResult::Next
        },
    );

    app.listen("127.0.0.1:5050");
}
