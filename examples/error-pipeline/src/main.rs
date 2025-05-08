use std::{fs, io};
use feather::{next, App, AppContext, Request, Response};
/// Lets see how can we use the new error pipeline feature!
fn main() {
    let mut app = App::new();
    app.get("/", |_req: &mut Request,_res: &mut Response,_ctx: &mut AppContext|{
        // Lets say we have a operation that can fail for this example a File Access
        let _file: fs::File = fs::File::open("file.txt")?; // With the ? Operator we can easily toss the error in the pipeline to be handled
        next!()
    });

    // if there is no Custom Error handler set Framework will catch the error log it and send a 500 back to the client
    // We can attach a custom error handler with this function
    app.set_error_handler(Box::new(|err,_req,res|{
        println!("A Error Accured");
        if err.is::<io::Error>(){
            println!("Error is a IO error{err}");
            res.set_status(500).send_text("Missing data on the server? Internal Error");
        }
    }));
    // This way we can handle Errors Gracefully and safely.
    app.listen("127.0.0.1:5050");
}