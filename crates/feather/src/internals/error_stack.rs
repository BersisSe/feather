// I get it its kinda pointess to open a new module for just a 2 types but maybe I'll add more features to the errors ;)

use feather_runtime::http::{Request, Response};
use std::error::Error;

type BoxError = Box<dyn Error>;

/// Type Alias for the Error Handling Function: `Box<dyn Fn(BoxError,&Request,&mut Response)>`
pub type ErrorHandler = Box<dyn Fn(BoxError, &Request, &mut Response)>;
