use std::collections::HashMap;
use std::sync::Arc;

use feather_runtime::http::Request;
use feather_runtime::http::Response;
use feather_runtime::runtime::MayStream;
use feather_runtime::runtime::Service;
use feather_runtime::runtime::service::ServiceResult;

use crate::AppContext;
use crate::internals::app::Route;
use crate::internals::error_stack::ErrorHandler;
use crate::middlewares::Middleware;

pub(crate) struct AppService {
    pub routes: Vec<Route>,
    pub middleware: Vec<Arc<dyn Middleware>>,
    pub context: AppContext,
    pub error_handler: Option<ErrorHandler>,
}

impl AppService {
    fn run_middleware(mut request: &mut Request, routes: &[Route], global_middleware: &[Arc<dyn Middleware>], context: &AppContext, error_handler: &Option<ErrorHandler>) -> Response {
        let mut response = Response::default();
        // Run global middleware

        for middleware in global_middleware {
            match middleware.handle(&mut request, &mut response, &context) {
                Ok(crate::middlewares::MiddlewareResult::Next) => {}
                Ok(crate::middlewares::MiddlewareResult::NextRoute) => break,
                Ok(crate::middlewares::MiddlewareResult::End) => return response,
                Err(e) => {
                    if let Some(handler) = &error_handler {
                        handler(e, &request, &mut response)
                    } else {
                        eprintln!("Unhandled Error caught in middlewares: {}", e);
                        response.set_status(500).send_text("Internal Server Error!");
                        return response;
                    }
                }
            }
        }
        let method = request.method.clone();
        // Run route-specific middleware
        let mut found = false;
        for route in routes.iter().filter(|r| r.method == method) {
            if let Some(params) = Self::match_route(&route.path, &request.path()) {
                request.set_params(params);
                match route.middleware.handle(request, &mut response, &context) {
                    Ok(crate::middlewares::MiddlewareResult::NextRoute) => {
                        // Skip this match and keep looking for the next matching route
                        continue;
                    }
                    Ok(crate::middlewares::MiddlewareResult::End) | Ok(crate::middlewares::MiddlewareResult::Next) => {
                        found = true;
                        break;
                    }
                    Err(e) => {
                        if let Some(handler) = &error_handler {
                            handler(e, &request, &mut response)
                        } else {
                            eprintln!("Unhandled Error caught in Route Middlewares : {}", e);
                            response.set_status(500).send_text("Internal Server Error");
                            break;
                        }
                    }
                }
            }
        }
        if !found {
            response.set_status(404).send_text("404 Not Found");
        }

        response
    }
    fn match_route<'r>(pattern: &'r str, path: &'r str) -> Option<HashMap<String, String>> {
        let mut params = HashMap::new();
        let pattern_parts: Vec<&str> = pattern.trim_matches('/').split('/').collect();
        let path_parts: Vec<&str> = path.trim_matches('/').split('/').collect();

        if pattern_parts.len() != path_parts.len() {
            return None;
        }

        for (pat, val) in pattern_parts.iter().zip(path_parts.iter()) {
            if pat.starts_with(':') {
                params.insert(pat[1..].to_string(), val.to_string());
            } else if pat != val {
                return None;
            }
        }

        Some(params)
    }
}

impl Service for AppService {
    fn handle(&self, mut req: feather_runtime::http::Request, _stream: Option<MayStream>) -> std::io::Result<ServiceResult> {
        let response = Self::run_middleware(&mut req, &self.routes, &self.middleware, &self.context, &self.error_handler);
        return Ok(ServiceResult::Response(response));
    }
}
