// This example demonstrates how to use JWT authentication in a Feather application.
use feather::jwt::{generate_jwt, with_jwt_auth};
use feather::{App, AppContext, next};

fn main() {
    // Create a new instance of App
    let mut app = App::new();
    // This route will used to generate a JWT token
    // and send it back to the client
    // The token will be valid for 1 hour
    // and will be signed with the secret
    app.get("/", |_req: &mut feather::Request, res: &mut feather::Response, ctx: &mut AppContext| {
        let token = generate_jwt(Some("Subject"), "secretcode").unwrap();
        res.send_text(format!("Token: {}", token));
        ctx.set_state(token);
        next!()
    });
    // This route will be used to test the JWT authentication
    // It will check if the token is valid and not expired
    // If the token is valid, it will send back a message(Hello, JWT!)
    // If the token is invalid or expired, it will send back a 401 error

    app.get(
        "/auth",
        with_jwt_auth("secretcode", |_req, res, _ctx, claim| {
            println!("Claim: {:?}", claim);
            let token = _ctx.get_state::<String>();
            println!("Toke: {}", token.unwrap());
            res.send_text("Hello, JWT!");
            next!()
        }),
    );
    // Of course lets listen on port 8080
    app.listen("127.0.0.1:8080")
}
