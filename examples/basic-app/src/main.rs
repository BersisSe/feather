use feather::{App, AppContext, Outcome, Request, Response, middleware_fn, next};
use std::fs;
/// adada
fn main() {
    let mut app = App::new();
    app.get("/", api_route);
    let a = 5;
    
}


#[middleware_fn]
fn api_route(){
    let f = fs::File::open("a.txt");
    
    next!()
}