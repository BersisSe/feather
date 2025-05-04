use feather::{App, AppContext, MiddlewareResult, Request, Response};

fn main() {
    let mut app = App::new();

    app.get("/", |_req: &mut Request, res: &mut Response, _ctx: &mut AppContext| {
        res.send_bytes([]);
        MiddlewareResult::NextRoute
    });

    app.post("/user", |_req: &mut Request, _res: &mut Response, _ctx: &mut AppContext| {
        MiddlewareResult::NextRoute
    });

    app.get("/user", |req: &mut Request, res: &mut Response, _ctx: &mut AppContext| {
        if let Some(query) = req.uri.query() {
            res.send_bytes(query);       
        } else {
            res.send_bytes([]);
        }
        
        MiddlewareResult::NextRoute
    });

    app.listen("0.0.0.0:3000");
}
