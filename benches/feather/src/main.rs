use feather::{App, AppContext, next, Request, Response, middleware};

fn main() {
    let mut app = App::new();

    app.get("/", middleware!(|_req, res, _ctx| {
        res.send_bytes([]);
        next!()
    }));

    app.post("/user", middleware!(|_req, _res, _ctx| {
        next!()
    }));

    app.get("/user", middleware!(|req, res, _ctx| {
        if let Some(query) = req.uri.query() {
            res.send_bytes(query);       
        } else {
            res.send_bytes([]);
        }
        next!()
    }));

    app.listen("0.0.0.0:3000");
}
