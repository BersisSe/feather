use crate::{AppContext, Outcome, Request, Response, middlewares::Middleware, next};
use jsonwebtoken::{encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
pub use jsonwebtoken::errors::ErrorKind;
pub use jsonwebtoken::errors::Error;
/// Represents a claim that can be validated after decoding the token
/// You can override `validate` if you want to check required fields or expiry etc.
pub trait Claim: DeserializeOwned {
    fn validate(&self) -> Result<(), Error> {
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub struct SimpleClaims {
    pub sub: String,
    pub exp: usize,
}

impl Claim for SimpleClaims {
    fn validate(&self) -> Result<(), Error> {
        if self.sub.is_empty() {
            return Err(Error::from(jsonwebtoken::errors::ErrorKind::InvalidToken));
        }
        if self.exp < ::std::time::SystemTime::now()
            .duration_since(::std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize
        {
            return Err(Error::from(jsonwebtoken::errors::ErrorKind::ExpiredSignature));
        }
        Ok(())
    }
}

/// Helper struct to encode/decode JWT tokens using a shared secret
#[derive(Debug)]
pub struct JwtManager {
    secret: String,
}

impl JwtManager {
    /// Create a new JwtManager with a secret key
    pub fn new(secret: String) -> Self {
        Self { secret }
    }

    /// Decode and validate a token into claims
    pub fn decode<T: for<'de> Deserialize<'de> + Claim>(&self, token: &str)
        -> Result<T, jsonwebtoken::errors::Error>
    {
        let data = jsonwebtoken::decode::<T>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::default(),
        )?;
        data.claims.validate()?;
        Ok(data.claims)
    }

    /// Encode a claims object into a token
    pub fn encode<T: Serialize>(&self, claims: &T) -> Result<String, jsonwebtoken::errors::Error> {
        encode(
            &Header::default(),
            claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
    }

    /// Creates a basic token with only `sub` and `exp` fields
    /// This is useful for quick usage without defining a full claim struct
    pub fn generate_simple(&self, subject: &str, ttl_hours: i64) -> Result<String, jsonwebtoken::errors::Error> {
        let claims = SimpleClaims {
            sub: subject.to_owned(),
            exp: chrono::Utc::now()
                .checked_add_signed(chrono::Duration::hours(ttl_hours))
                .unwrap()
                .timestamp() as usize,
        };

        self.encode(&claims)
    }
}

/// Protects a route using JWT authentication
///
/// This middleware checks for a valid `Authorization: Bearer <token>` header,
/// decodes it using the [JwtManager] in the [AppContext], and passes the claims
/// to the handler function.
///
/// If token is invalid or expired, a 401 error is returned.
pub fn with_jwt_auth<T, F>(handler: F) -> impl Middleware
where
    T: for<'de> serde::de::Deserialize<'de> + Claim + 'static,
    F: Fn(&mut Request, &mut Response, &mut AppContext, T) -> Outcome,
{
    move |req: &mut Request, res: &mut Response, ctx: &mut AppContext| -> Outcome {
        let manager = ctx.jwt();
        let token = match req
            .headers
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
        {
            Some(t) => t,
            None => {
                res.set_status(401);
                res.send_text("Missing or invalid Authorization header");
                return next!();
            }
        };

        let claims: T = match manager.decode(token) {
            Ok(c) => c,
            Err(_) => {
                res.set_status(401);
                res.send_text("Invalid or expired token");
                return next!();
            }
        };

        handler(req, res, ctx, claims)
    }
}
