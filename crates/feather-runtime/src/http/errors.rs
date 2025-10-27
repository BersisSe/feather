use http::header::{InvalidHeaderName,InvalidHeaderValue};
use thiserror::Error;


#[derive(Debug,Error)]
pub enum HeaderError {
    #[error("Invalid Header Name")]
    InvalidHeaderName(#[from] InvalidHeaderName),
    #[error("Invalid Header Value")]
    InvalidHeaderValue(#[from] InvalidHeaderValue),
}