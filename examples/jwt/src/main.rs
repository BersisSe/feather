// This example demonstrates how to use JWT authentication in a Feather application.

use feather::jwt::Claim;
use feather::jwt::{self, JwtManager, with_jwt_auth};
use feather::jwt_required;
use feather::{App, AppContext, Claim, middleware, next};
use serde::{Deserialize, Serialize};

fn main() {
    // Create a new instance of App
    let mut app = App::new();
    // Lets create a manager this will keep our secret.
    let manager = JwtManager::new("top-secret-key".to_string());
    // If Jwt Feature is active Context gains some new perks
    app.context().set_jwt(manager);
    // This route will used to generate a JWT token
    // and send it back to the client
    // The token will be valid for 1 hour
    // and will be signed with the secret
    app.get("/token1", |_req: &mut feather::Request, res: &mut feather::Response, ctx: &AppContext| {
        // With the use this function we can generate a token with ease
        let token = ctx.jwt().generate_simple("AppAuth", 1)?;
        res.send_text(format!("Token: {}", token));
        ctx.set_state(token);
        next!()
    });
    app.post(
        "/token2",
        middleware!(|req, res, ctx| {
            let name = req.json()?.get("name").unwrap_or(&feather::Value::String("No Name".into())).to_string();
            let exp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as usize + 3600;
            let claims = MyClaim {
                name,
                exp,
                sub: "Named Token".into(),
            };
            let token = ctx.jwt().encode(&claims)?;
            res.send_text(token);
            next!()
        }),
    );

    // This route will be used to test the JWT authentication
    // It will check if the token is valid and not expired
    // If the token is valid, it will send back a message(Hello, JWT!)
    // If the token is invalid or expired, it will send back a 401 error
    app.get(
        "/protected1",
        with_jwt_auth::<jwt::SimpleClaims, _>(|_req, res, _ctx, claims| {
            res.send_text(format!("Hello Your Subject: {}", claims.sub));
            next!()
        }),
    );

    app.get("/protected2", protected2);

    // Of course lets listen on port 5050
    app.listen("127.0.0.1:5050")
}

// You can Also Create your own claims with diffent fields or even methods
// Derive Claim trait to use it with jwt_required macro
#[derive(Claim, Deserialize, Serialize)]
struct MyClaim {
    #[exp]
    exp: usize,
    #[required]
    sub: String,
    #[required]
    name: String,
}

// With the use of jwt_required macro we can protect our routes
#[jwt_required]
#[middleware_fn]
fn protected2(claims: MyClaim) -> feather::Outcome {
    res.send_text(format!("Hello {}", claims.name));
    next!()
}
