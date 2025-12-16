//! This example demonstrates how to create a simple API using Feather.
//! It includes a GET handler for the root path and a POST handler for user authentication.
//! //! The POST handler expects a JSON body with a "username" field and responds accordingly.
//! //! The server listens on port 5050.

use feather::{App, info, json, middleware, middleware_fn, next};

fn main() {
    // Lets Create a App instance named api
    let mut api = App::new();
    
    // Register the get_handler function for the "/" path
    api.get("/", get_handler);

    // Lets use a post handler to simulate a login
    // This will be called when a POST request is made to the "/auth" path
    // The data will be parsed as JSON and echoed back to the client
    api.post(
        "/auth",
        middleware!(|req, res, _ctx| {
            let data = req.json()?; // Propogate error to the pipeline
            // Log the received data
            info!("Received POST request to /auth with body: {}", data);
            // Check if the data contains a "username" field
            match data.get("username") {
                Some(username) => {
                    // If the username is present, send a 200 OK response with the username
                    info!("Username: {}", username);
                    res.set_status(200).send_json(json!({
                        "message": "Login successful",
                        "username": username
                    }));
                    next!()
                }
                None => {
                    // If the username is not present, send a 400 Bad Request response
                    res.set_status(400).send_json(json!({
                        "error": "Username is required"
                    }));
                    return next!();
                }
            }
        }),
    );
    // We have to listen to the api instance to start the server
    // This will start the server on port 5050
    api.listen("127.0.0.1:5050");
}

// Handler Can Also Be Functions Like this
// This function will be called when a GET request is made to the "/"
#[middleware_fn]
fn get_handler() {
    
    res.send_html("<h1>Hello I am an Feather Api</h1>");
    next!()
}
