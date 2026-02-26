#[cfg(feature = "async")]
mod bridge_impl {
    use feather::{info, next, Request, Response, AppContext, Outcome, MiddlewareResult};
    use std::time::Duration;
    use serde::{Deserialize, Serialize};
    use async_std::future::timeout;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[derive(feather::Claim, Clone, Serialize, Deserialize)]
    pub struct UserClaims {
        #[required]
        #[serde(rename = "sub")]
        pub user_id: String,
    }

    pub struct AsyncDataMiddleware {
        pub call_count: AtomicUsize,
    }

    impl AsyncDataMiddleware {
        // Simulates an I/O bound permission check with transient failure.
        pub async fn fetch_user_permissions(&self, user_id: &str) {
            let count = self.call_count.fetch_add(1, Ordering::SeqCst);
            
            info!("Fetching permissions for UID: {} (Attempt {})", user_id, count + 1);

            // Simulate high latency on the first call to trigger the bridge timeout logic
            if count == 0 {
                async_std::task::sleep(Duration::from_millis(800)).await;
            } else {
                async_std::task::sleep(Duration::from_millis(200)).await;
            }
            
            info!("Permissions loaded for UID: {}", user_id);
        }
    }

    impl feather::middlewares::Middleware for AsyncDataMiddleware {
        fn handle(&self, req: &mut Request, res: &mut Response, ctx: &AppContext) -> Outcome {
            if req.path() == "/token" { return next!(); }

            let manager = ctx.jwt();
            let token = match req.headers.get("Authorization")
                .and_then(|h| h.to_str().ok())
                .and_then(|h| h.strip_prefix("Bearer ")) 
            {
                Some(t) => t,
                None => {
                    res.set_status(401);
                    res.send_text("Unauthorized");
                    return Ok(MiddlewareResult::End); 
                }
            };

            let claims: UserClaims = match manager.decode(token) {
                Ok(c) => c,
                Err(_) => {
                    res.set_status(401);
                    res.send_text("Invalid Session");
                    return Ok(MiddlewareResult::End);
                }
            };

            // BRIDGE: Execute async I/O within the synchronous middleware chain.
            // This uses a retry loop with a hard timeout per attempt to ensure 
            // request-line resilience without blocking the OS thread.
            feather::runtime::executor::block_on(async {
                let mut attempts = 0;
                let max_retries = 3;

                loop {
                    attempts += 1;
                    
                    match timeout(Duration::from_millis(600), self.fetch_user_permissions(&claims.user_id)).await {
                        Ok(_) => return next!(),
                        Err(_) => {
                            if attempts >= max_retries {
                                info!("Critical failure: UID {} timed out after {} retries", claims.user_id, attempts);
                                res.set_status(504);
                                res.send_text("Gateway Timeout");
                                return Ok(MiddlewareResult::End);
                            }
                            
                            info!("Transient error for UID {}; retrying ({}/{})", claims.user_id, attempts, max_retries);
                            async_std::task::sleep(Duration::from_millis(100)).await;
                        }
                    }
                }
            })
        }
    }
}

#[cfg(feature = "async")]
pub use bridge_impl::*;