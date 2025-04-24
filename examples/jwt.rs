// This example demonstrates how to use JWT authentication in a Feather application.
use feather::App;
use feather::jwt::{generate_jwt, with_jwt_auth};

fn main() {
    // Create a new instance of App
    let mut app = App::new();
    // This route will used to generate a JWT token
    // and send it back to the client
    // The token will be valid for 1 hour
    // and will be signed with the secret
    app.get("/", |req: &mut feather::Request, res: &mut feather::Response| {
        let token = generate_jwt(Some("A"), "secretcode").unwrap();
        let tk = token.clone();
        req.extensions.insert(token);
        res.send_text(format!("Token: {}", tk));
        feather::MiddlewareResult::Next
    });
    // This route will be used to test the JWT authentication
    // It will check if the token is valid and not expired
    // If the token is valid, it will send back a message(Hello, JWT!)
    // If the token is invalid or expired, it will send back a 401 error

    app.get("/auth", with_jwt_auth("secretcode", |_req,res, claim| {
        println!("Claim: {:?}", claim);
        res.send_text("Hello, JWT!");
        feather::MiddlewareResult::Next
    }));
    // Of course lets listen on port 8080
    app.listen("127.0.0.1:8080")
}