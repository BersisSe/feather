use feather::{App, next, Request, Response, AppContext};
use feather::jwt::JwtManager;

mod async_logic;

fn main() {
    let mut app = App::new();

    let jwt_manager = JwtManager::new("secret-key".to_string());
    app.context().set_jwt(jwt_manager);

    // --- MIDDLEWARE REGISTRATION ---
    
    #[cfg(feature = "async")]
    {
        use std::sync::atomic::AtomicUsize;
        use async_logic::AsyncDataMiddleware;

        let middleware = AsyncDataMiddleware {
            call_count: AtomicUsize::new(0),
        };
        app.use_middleware(middleware);
    }

    #[cfg(not(feature = "async"))]
    app.use_middleware(|_req: &mut Request, _res: &mut Response, _ctx: &AppContext| {
        feather::info!("Running in pure Synchronous mode (Async Bridge Disabled)");
        next!()
    });

    // --- ROUTE REGISTRATION ---

    #[cfg(feature = "async")]
    app.get("/", delayed_welcome);

    #[cfg(not(feature = "async"))]
    app.get("/", |_req: &mut Request, res: &mut Response, _ctx: &AppContext| {
        res.send_text("Welcome! You are currently using the feather-sync baseline.");
        next!()
    });

    // Public endpoint for JWT generation
    app.get("/token", |_req: &mut Request, res: &mut Response, ctx: &AppContext| {
        let token = ctx.jwt().generate_simple("12345", 24).unwrap();
        res.send_text(token);
        next!()
    });

    // Protected endpoint
    app.get("/secure", |_req: &mut Request, res: &mut Response, _ctx: &AppContext| {
        res.send_text("Authorized access: You reached the final handler!");
        next!()
    });

    println!("Starting Feather on http://127.0.0.1:5050");
    app.listen("127.0.0.1:5050");
}

#[cfg(feature = "async")]
#[feather::middleware_fn]
async fn delayed_welcome(_req: Request, res: Response, _ctx: AppContext) -> feather::Outcome {
    feather::info!("Async function middleware started...");
    async_std::task::sleep(std::time::Duration::from_secs(1)).await;
    res.send_text("Hello from the Async-Bridged world!");
    next!()
}