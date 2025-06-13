use feather::{next, App, AppContext, Request, Response};

fn main() {
    let mut app = App::new();
    // Define a Route With a Dynamic Path
    app.get("/users/:id", |req: &mut Request,res: &mut Response,_ctx: &mut AppContext|{
        let id = req.param("id");
        res.send_text(format!("Welcome User: {}",id.unwrap()));
        next!()
    });
    
    //Lets Listen on port 5050
    app.listen("127.0.0.1:5050");
}
