use proc_macro::TokenStream;
use quote::quote;
#[cfg(feature = "jwt")]
use syn::{Data, DeriveInput, Fields};
use syn::{ItemFn, parse_macro_input};

/// This macro derives the `Claim` trait for a struct, allowing it to be used as JWT claims.
/// It checks for fields annotated with `#[required]` and `#[exp]` to
/// validate the claims when decoding a JWT token.
/// The `#[required]` attribute ensures that the field is not empty, and the `#[exp]` attribute checks if the field is a valid expiration time.
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

/// This macro defines a middleware function that can be used in Feather applications.  
/// It allows you to write middleware functions without repeating the type signatures for request, response, and context.
/// Example:
/// ```rust,ignore
/// use feather::{middleware_fn, Outcome, next};
/// #[middleware_fn]
/// fn my_middleware() -> Outcome {
///     res.send_text("Hello from middleware!");
///     next!()
/// }
///     // Your middleware logic here
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

/// This macro is used to define a JWT-required middleware function.
/// It expects a function with a specific signature that includes a claims argument.
/// The claims argument must implement the `feather::jwt::Claim` trait.
/// Example:
/// ```rust,ignore
/// use feather::{jwt_required, middleware_fn, Outcome, next};
/// use feather::jwt::{JwtManager, SimpleClaims};
/// #[jwt_required]
/// #[middleware_fn]
/// fn protected_route(claims: SimpleClaims) -> Outcome {
///   // Your Logic Here
///   next!()
/// }
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
