use feather::{next, App, AppContext, Outcome, Request, Response};

fn main() {
    simple_logger::init();
    let mut app = App::new();
    
    app.ws("/chat",|ws|{
        ws.on_message(|c,msg|{
            c.send("Hi");
        });
    });
    
    app.listen("127.0.0.1:5050");
}

