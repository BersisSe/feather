use feather::{middleware::Middleware, next};

// Middlewares can be any type that implements the `Middleware` Trait
pub struct MyMiddleware(pub String);

// All middlewares have access to the app context and request/response objects
impl Middleware for MyMiddleware {
    fn handle(
        &self,
        _request: &mut feather::Request,
        _response: &mut feather::Response,
        _ctx: &mut feather::AppContext,
    ) ->  feather::Outcome {
        // Structs also have the `self` parameter but its behind a non mutuable referance so you cant just mutate the struct
        println!("Hii I am a Struct Middleware and this is my data: {}",self.0);

        next!()
    }
}
