use proc_macro::TokenStream;
use quote::quote;
#[cfg(feature = "jwt")]
use syn::{Data, DeriveInput, Fields};
use syn::{ItemFn, parse_macro_input};

/// Derive macro for implementing the `Claim` trait for JWT claims.
///
/// This macro automatically derives the `Claim` trait for your claims struct,
/// enabling validation of required fields and JWT expiration times.
///
/// # Attributes
///
/// - `#[required]` - Mark a field as required (must not be empty)
/// - `#[exp]` - Mark a field as the expiration timestamp (checks against current time)
///
/// # Example: Simple Claims
///
/// ```rust,ignore
/// use feather::jwt::Claim;
///
/// #[derive(Claim, Clone)]
/// struct MyClaims {
///     user_id: String,
///     username: String,
/// }
/// ```
///
/// # Example: With Validation
///
/// ```rust,ignore
/// use feather::jwt::Claim;
///
/// #[derive(Claim, Clone)]
/// struct AuthClaims {
///     #[required]
///     user_id: String,
///     #[required]
///     username: String,
/// }
/// ```
///
/// # Example: With Expiration
///
/// ```rust,ignore
/// use feather::jwt::Claim;
///
/// #[derive(Claim, Clone)]
/// struct TokenClaims {
///     #[required]
///     user_id: String,
///     #[exp]
///     expires_at: usize,  // Unix timestamp
/// }
/// ```
///
/// # How It Works
///
/// The macro generates a `validate()` method that:
/// 1. Checks all `#[required]` fields are non-empty
/// 2. Checks `#[exp]` fields contain timestamps greater than current time
/// 3. Returns `Err` if any validation fails
///
/// This is automatically called by the JWT manager when decoding tokens.
///
/// # See Also
///
/// - [`SimpleClaims`](https://docs.rs/feather/latest/feather/jwt/struct.SimpleClaims.html) for a built-in claims struct
/// - [Authentication Guide](https://docs.rs/feather/latest/feather/guides/authentication/) for JWT patterns
#[cfg(feature = "jwt")]
#[proc_macro_derive(Claim, attributes(required, exp))]
pub fn derive_claim(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let mut checks = Vec::new();

    if let Data::Struct(data_struct) = &input.data {
        if let Fields::Named(fields) = &data_struct.fields {
            for field in &fields.named {
                let field_name = &field.ident;
                for attr in &field.attrs {
                    if attr.path().is_ident("required") {
                        checks.push(quote! {
                            if self.#field_name.is_empty() {
                                return Err(feather::jwt::Error::from(feather::jwt::ErrorKind::InvalidToken));
                            }
                        });
                    }
                    if attr.path().is_ident("exp") {
                        checks.push(quote! {
                            if self.#field_name < ::std::time::SystemTime::now().duration_since(::std::time::UNIX_EPOCH).unwrap().as_secs() as usize {
                                return Err(feather::jwt::Error::from(feather::jwt::ErrorKind::ExpiredSignature));
                            }
                        });
                    }
                }
            }
        }
    }

    let expanded = quote! {
        impl feather::jwt::Claim for #name {
            fn validate(&self) -> Result<(), feather::jwt::Error> {
                #(#checks)*
                Ok(())
            }
        }
    };
    TokenStream::from(expanded)
}

/// Attribute macro for defining middleware functions with automatic signature injection.
///
/// This macro eliminates boilerplate by automatically providing `req`, `res`, and `ctx` parameters
/// to your middleware function. It transforms a simple function into a proper Feather middleware.
///
/// # What This Macro Does
///
/// The `#[middleware_fn]` macro injects three parameters into your function:
/// - `req: &mut Request` - The HTTP request
/// - `res: &mut Response` - The HTTP response
/// - `ctx: &AppContext` - Application context for accessing state
///
/// Your function must return `Outcome` (which is `Result<MiddlewareResult, Box<dyn Error>>`).
///
/// # Basic Example
///
/// ```rust,ignore
/// use feather::middleware_fn;
///
/// #[middleware_fn]
/// fn log_requests() {
///     println!("{} {}", req.method, req.uri);
///     next!()
/// }
///
/// app.use_middleware(log_requests);
/// ```
///
/// # With Route Handlers
///
/// ```rust,ignore
/// use feather::{App, middleware_fn};
///
/// #[middleware_fn]
/// fn greet() {
///     let name = req.param("name").unwrap_or("Guest".to_string());
///     res.send_text(format!("Hello, {}!", name));
///     next!()
/// }
///
/// let mut app = App::new();
/// app.get("/greet/:name", greet);
/// ```
///
/// # Compared to the middleware! Macro
///
/// Both `#[middleware_fn]` and `middleware!` work similarly, but `#[middleware_fn]` is
/// best for reusable, named middleware functions, while `middleware!` is best for inline closures:
///
/// ```rust,ignore
/// // Using #[middleware_fn] - for reusable middleware
/// #[middleware_fn]
/// fn validate_auth() {
///     if !req.headers.contains_key("Authorization") {
///         res.set_status(401);
///         res.send_text("Unauthorized");
///         return next!();
///     }
///     next!()
/// }
///
/// app.use_middleware(validate_auth);
///
/// // Using middleware! - for inline middleware
/// app.get("/", middleware!(|_req, res, _ctx| {
///     res.send_text("Hello!");
///     next!()
/// }));
/// ```
///
/// # Accessing Application State
///
/// ```rust,ignore
/// use feather::{State, middleware_fn};
///
/// #[derive(Clone)]
/// struct Config {
///     api_key: String,
/// }
///
/// #[middleware_fn]
/// fn check_api_key() {
///     let config = ctx.get_state::<State<Config>>();
///     let is_valid = config.with_scope(|cfg| cfg.api_key == "secret");
///     
///     if !is_valid {
///         res.set_status(403);
///         res.send_text("Forbidden");
///         return next!();
///     }
///     next!()
/// }
/// ```
///
/// # Error Handling
///
/// ```rust,ignore
/// use feather::middleware_fn;
///
/// #[middleware_fn]
/// fn parse_json() {
///     if let Ok(body) = String::from_utf8(req.body.clone()) {
///         // Process body
///         next!()
///     } else {
///         res.set_status(400);
///         res.send_text("Invalid UTF-8");
///         next!()
///     }
/// }
/// ```
///
/// # See Also
///
/// - Use `#[jwt_required]` together with `#[middleware_fn]` for JWT-protected routes
/// - See the [Middlewares Guide](https://docs.rs/feather/latest/feather/guides/middlewares/) for more patterns
#[proc_macro_attribute]
pub fn middleware_fn(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let vis = &input.vis;
    let sig: &syn::Signature = &input.sig;
    let block = &input.block;
    let fn_name = &sig.ident;

    let expanded = quote! {
        #vis fn #fn_name(
            req: &mut feather::Request,
            res: &mut feather::Response,
            ctx: &feather::AppContext
        ) -> feather::Outcome {
            #block
        }
    };
    TokenStream::from(expanded)
}

/// Attribute macro for creating JWT-protected middleware.
///
/// Combines with `#[middleware_fn]` to automatically extract and validate JWT claims
/// from the `Authorization` header. Only works with `#[middleware_fn]`.
///
/// # How It Works
///
/// This macro:
/// 1. Extracts the JWT token from the `Authorization: Bearer <token>` header
/// 2. Decodes and validates the token using the app's JWT manager
/// 3. Validates claims using the `Claim` trait
/// 4. Injects the decoded claims into your function
///
/// If any step fails, it returns a 401 Unauthorized response automatically.
///
/// # Syntax
///
/// ```rust,ignore
/// #[jwt_required]
/// #[middleware_fn]
/// fn your_handler(claims: YourClaimsType) {
///     // claims are now available
///     next!()
/// }
/// ```
///
/// # Example: Protecting a Route
///
/// ```rust,ignore
/// use feather::{jwt_required, middleware_fn, Claim};
///
/// #[derive(Claim, Clone)]
/// struct AuthClaims {
///     #[required]
///     user_id: String,
///     username: String,
/// }
///
/// #[jwt_required]
/// #[middleware_fn]
/// fn protected_profile() {
///     res.send_text(format!("Profile for: {}", claims.username));
///     next!()
/// }
///
/// let mut app = App::new();
/// app.get("/profile", protected_profile);
/// ```
///
/// # Example: With SimpleClaims
///
/// ```rust,ignore
/// use feather::{jwt_required, middleware_fn};
/// use feather::jwt::SimpleClaims;
///
/// #[jwt_required]
/// #[middleware_fn]
/// fn get_user() {
///     res.send_text(format!("User: {}", claims.sub));
///     next!()
/// }
/// ```
///
/// # Example: Accessing Claim Fields
///
/// ```rust,ignore
/// #[jwt_required]
/// #[middleware_fn]
/// fn protected_route(claims: AuthClaims) {
///     // Access claim fields
///     let user_id = &claims.user_id;
///     let username = &claims.username;
///     
///     // Store in response or context
///     ctx.set_state(State::new(user_id.clone()));
///     res.send_text(format!("Welcome, {}!", username));
///     next!()
/// }
/// ```
///
/// # Integration with the App
///
/// Remember to configure the JWT manager:
/// ```rust,ignore
/// use feather::App;
/// use feather::jwt::JwtManager;
///
/// let mut app = App::new();
/// let jwt_manager = JwtManager::new("your-secret-key");
/// app.context().set_state(State::new(jwt_manager));
/// ```
///
/// # Error Handling
///
/// Automatic 401 responses are sent if:
/// - `Authorization` header is missing or malformed
/// - Token is invalid or expired
/// - Claims fail validation
///
/// To customize error responses, use `#[middleware_fn]` with manual JWT handling.
///
/// # See Also
///
/// - [`#[middleware_fn]`](attr.middleware_fn.html) - The companion macro required with `#[jwt_required]`
/// - [`JwtManager`](https://docs.rs/feather/latest/feather/jwt/struct.JwtManager.html) - JWT token management
/// - [Authentication Guide](https://docs.rs/feather/latest/feather/guides/authentication/) - JWT patterns and examples
#[cfg(feature = "jwt")]
#[proc_macro_attribute]
pub fn jwt_required(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;
    let vis = &input.vis;
    let block = &input.block;
    let inputs = &input.sig.inputs;

    let claims_ident = inputs.iter().find_map(|arg| {
        if let syn::FnArg::Typed(pat_type) = arg {
            if let syn::Pat::Ident(ident) = &*pat_type.pat {
                Some((&ident.ident, &*pat_type.ty))
            } else {
                None
            }
        } else {
            None
        }
    });

    let (claims_name, claims_type) = match claims_ident {
        Some(x) => x,
        None => {
            return syn::Error::new_spanned(&input.sig, "expected a `claims: T` argument for #[jwt_required]").to_compile_error().into();
        }
    };

    let expanded = quote! {
        #vis fn #fn_name(req: &mut feather::Request, res: &mut feather::Response, ctx: &feather::AppContext) -> feather::Outcome {
            let manager = ctx.jwt();
            let token = match req
                .headers
                .get("Authorization")
                .and_then(|h| h.to_str().ok())
                .and_then(|h| h.strip_prefix("Bearer ")) {
                    Some(t) => t,
                    None => {
                        res.set_status(401);
                        res.send_text("Missing or invalid Authorization header");
                        return feather::next!();
                    }
                };

            let #claims_name: #claims_type = match manager.decode(token) {
                Ok(c) => c,
                Err(_) => {
                    res.set_status(401);
                    res.send_text("Invalid or expired token");
                    return feather::next!();
                }
            };

            if let Err(_) = #claims_name.validate() {
                res.set_status(401);
                res.send_text("Invalid or expired token");
                return feather::next!();
            }

            #block
        }
    };

    TokenStream::from(expanded)
}
