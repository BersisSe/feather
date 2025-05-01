use crate::{middleware::Middleware, AppContext, MiddlewareResult, Request, Response}; // adapt to your framework structure
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

/// Used to protect the route with JWT authentication
/// The middleware will check if the token is valid and not expired
/// If the token is valid, it will call the handler function
/// If the token is invalid or expired, it will send back a 401 error
/// The handler function will receive the request, response and the claims
pub fn with_jwt_auth<F>(secret: &'static str, handler: F) -> impl Middleware
where
    F: Fn(&mut Request, &mut Response,&mut AppContext, Claims) -> MiddlewareResult,
{
    move |req: &mut Request,
          res: &mut Response,
          ctx: &mut AppContext|
          -> MiddlewareResult {
        let Some(auth_header) = req.headers.get("Authorization") else {
            res.status(401);
            res.send_text("Missing Authorization header");
            return MiddlewareResult::NextRoute;
        };

        let Ok(auth_str) = auth_header.to_str() else {
            res.status(400);
            res.send_text("Invalid header format");
            return MiddlewareResult::NextRoute;
        };

        if !auth_str.starts_with("Bearer ") {
            res.status(400);
            res.send_text("Expected Bearer token");
            return MiddlewareResult::NextRoute;
        }

        let token = &auth_str[7..];

        match decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        ) {
            Ok(data) => handler(req, res,ctx, data.claims),
            Err(_) => {
                res.status(401);
                res.send_text("Invalid or expired token");
                MiddlewareResult::NextRoute
            }
        }
    }
}

/// Function to generate a JWT token
/// The token will be valid for 1 hour
/// and will be signed with the secret
pub fn generate_jwt(
    subject: Option<&str>,
    secret: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    let claims = Claims {
        sub: subject.unwrap_or_default().to_string(),
        exp: chrono::Utc::now()
            .checked_add_signed(chrono::Duration::hours(1))
            .expect("valid timestamp")
            .timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}
