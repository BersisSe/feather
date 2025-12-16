use crate::{AppContext, Outcome, Request, Response, middlewares::Middleware, next};
pub use jsonwebtoken::errors::Error;
pub use jsonwebtoken::errors::ErrorKind;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, encode};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

/// Trait for JWT claims validation.
///
/// Implement this trait for your claims struct to add custom validation logic
/// (e.g., checking expiration, required fields, permissions, etc.).
///
/// # Example
///
/// ```rust,ignore
/// use feather::jwt::Claim;
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Serialize, Deserialize)]
/// pub struct UserClaims {
///     pub sub: String,
///     pub exp: usize,
///     pub role: String,
/// }
///
/// impl Claim for UserClaims {
///     fn validate(&self) -> Result<(), Error> {
///         if self.role.is_empty() {
///             return Err(Error::from(ErrorKind::InvalidToken));
///         }
///         // Add more validation
///         Ok(())
///     }
/// }
/// ```
pub trait Claim: DeserializeOwned {
    /// Validate the claims. Return an error if invalid.
    fn validate(&self) -> Result<(), Error> {
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
/// Simple JWT claims with subject and expiration.
///
/// A basic claims struct for quick use without defining custom claims.
/// It includes standard claims for subject identification and expiration time.
///
/// # Fields
///
/// * `sub` - Subject (typically the user ID)
/// * `exp` - Expiration time as Unix timestamp
pub struct SimpleClaims {
    pub sub: String,
    pub exp: usize,
}

impl Claim for SimpleClaims {
    fn validate(&self) -> Result<(), Error> {
        if self.sub.is_empty() {
            return Err(Error::from(jsonwebtoken::errors::ErrorKind::InvalidToken));
        }
        let now = ::std::time::SystemTime::now()
            .duration_since(::std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize;
        if self.exp < now {
            return Err(Error::from(jsonwebtoken::errors::ErrorKind::ExpiredSignature));
        }
        Ok(())
    }
}

/// Helper for encoding and decoding JWT tokens with a shared secret.
///
/// `JwtManager` handles all JWT operations for your application. Create an instance
/// with your secret key and use it to generate and validate tokens.
///
/// # Example
///
/// ```rust,ignore
/// use feather::jwt::{JwtManager, SimpleClaims};
///
/// let jwt = JwtManager::new("your-secret-key".to_string());
///
/// // Generate a token
/// let token = jwt.generate_simple("user123", 24)?;
///
/// // Validate a token
/// let claims: SimpleClaims = jwt.decode(&token)?;
/// assert_eq!(claims.sub, "user123");
/// ```
#[derive(Debug, Clone)]
pub struct JwtManager {
    secret: String,
}

impl JwtManager {
    /// Create a new JWT manager with a secret key.
    ///
    /// # Arguments
    ///
    /// * `secret` - A cryptographically secure secret for signing/verifying tokens
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use feather::jwt::JwtManager;
    ///
    /// let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET not set");
    /// let jwt = JwtManager::new(secret);
    /// ```
    pub fn new(secret: String) -> Self {
        Self {
            secret,
        }
    }

    /// Decode and validate a token into claims of type `T`.
    ///
    /// # Arguments
    ///
    /// * `token` - The JWT token string
    ///
    /// # Returns
    ///
    /// The decoded claims if valid, or an error if invalid/expired.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use feather::jwt::SimpleClaims;
    ///
    /// let jwt = JwtManager::new("secret".to_string());
    /// match jwt.decode::<SimpleClaims>("token-string") {
    ///     Ok(claims) => println!("User: {}", claims.sub),
    ///     Err(e) => println!("Invalid token: {}", e),
    /// }
    /// ```
    pub fn decode<T: for<'de> Deserialize<'de> + Claim>(&self, token: &str) -> Result<T, jsonwebtoken::errors::Error> {
        let data = jsonwebtoken::decode::<T>(token, &DecodingKey::from_secret(self.secret.as_bytes()), &Validation::default())?;
        data.claims.validate()?;
        Ok(data.claims)
    }

    /// Encode claims into a JWT token.
    ///
    /// # Arguments
    ///
    /// * `claims` - The claims to encode (must implement Serialize)
    ///
    /// # Returns
    ///
    /// The JWT token string if successful.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use serde::{Serialize, Deserialize};
    /// use feather::jwt::JwtManager;
    ///
    /// #[derive(Serialize, Deserialize)]
    /// struct Claims {
    ///     sub: String,
    ///     exp: usize,
    /// }
    ///
    /// let jwt = JwtManager::new("secret".to_string());
    /// let token = jwt.encode(&Claims {
    ///     sub: "user123".to_string(),
    ///     exp: 1234567890,
    /// })?;
    /// ```
    pub fn encode<T: Serialize>(&self, claims: &T) -> Result<String, jsonwebtoken::errors::Error> {
        encode(&Header::default(), claims, &EncodingKey::from_secret(self.secret.as_bytes()))
    }

    /// Generate a simple token with subject and time-to-live.
    ///
    /// This is a convenience method for quick token generation without defining
    /// a custom claims struct.
    ///
    /// # Arguments
    ///
    /// * `subject` - The subject (usually user ID)
    /// * `ttl_hours` - Time to live in hours
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let jwt = JwtManager::new("secret".to_string());
    /// let token = jwt.generate_simple("user123", 24)?;  // Expires in 24 hours
    /// ```
    pub fn generate_simple(&self, subject: &str, ttl_hours: i64) -> Result<String, jsonwebtoken::errors::Error> {
        let claims = SimpleClaims {
            sub: subject.to_owned(),
            exp: chrono::Utc::now().checked_add_signed(chrono::Duration::hours(ttl_hours)).unwrap().timestamp() as usize,
        };

        self.encode(&claims)
    }
}

/// Protects a route using JWT authentication.
///
/// This middleware checks for a valid `Authorization: Bearer <token>` header,
/// decodes it using the `JwtManager` from the app context, and passes the claims
/// to the handler function.
///
/// Returns 401 Unauthorized if the token is missing, invalid, or expired.
///
/// # Arguments
///
/// * `handler` - Function that receives the request, response, context, and decoded claims
///
/// # Example
///
/// ```rust,ignore
/// use feather::jwt::{with_jwt_auth, SimpleClaims};
/// use feather::{App, next};
///
/// let mut app = App::new();
///
/// app.get("/protected", with_jwt_auth(|_req, res, _ctx, claims: SimpleClaims| {
///     res.send_text(format!("Hello, {}!", claims.sub));
///     next!()
/// }));
/// ```
pub fn with_jwt_auth<T, F: Send + Sync>(handler: F) -> impl Middleware
where
    T: for<'de> serde::de::Deserialize<'de> + Claim + 'static,
    F: Fn(&mut Request, &mut Response, &AppContext, T) -> Outcome,
{
    move |req: &mut Request, res: &mut Response, ctx: &AppContext| -> Outcome {
        let manager = ctx.jwt();
        let token = match req.headers.get("Authorization").and_then(|h| h.to_str().ok()).and_then(|h| h.strip_prefix("Bearer ")) {
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
