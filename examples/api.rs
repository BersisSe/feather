use feather::{Response,Request,App};

fn main() {
    // Lets Create a App instance named api
    let mut api = App::new();
    // Register the get_handler function for the "/" path 
    api.get("/", get_handler);
    // Lets use a post handler to simulate a login
    // This will be called when a POST request is made to the "/auth" path
    // The data will be parsed as JSON and returned as a response
    api.post("/auth", |req: &mut Request,res: &mut Response|{
        let data = req.json().unwrap();
        println!("Received data: {:?}", data);
        res.send_json(data);
        feather::MiddlewareResult::Next
    });   
    // We have to listen to the api instance to start the server
    // This will start the server on port 8080
    api.listen("127.0.0:8080");
}

// Handler Can Also Be Functions Like this
// This function will be called when a GET request is made to the "/"
fn get_handler(_req: &mut Request, res: &mut Response) -> feather::MiddlewareResult {
    res.send_html("<h1>Hello I am an Feather Api</h1>");
    feather::MiddlewareResult::Next
}   