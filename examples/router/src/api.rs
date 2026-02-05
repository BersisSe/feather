use feather::internals::Router;
use feather::{Finalizer, json, middleware, middleware_fn, next};

pub fn api_router() -> Router {
    let mut router = Router::new();

    // Middleware scoped only to this router
    router.use_middleware(middleware!(|_req, _res, _ctx| {
        println!("--> Scoped API Guard: Checking permissions...");
        next!()
    }));

    router.get("/data", get_data);

    router
}

#[middleware_fn]
fn get_data() {
    res.finish_json(&json!({ "status": "success", "data": [1, 2, 3] }))
}
