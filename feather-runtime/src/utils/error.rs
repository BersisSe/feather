use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("An IO error occurred: {0}")]
    IoError(#[from] std::io::Error),

    #[error("A parsing error occurred: {0}")]
    ParseError(String),

    #[error("An unknown error occurred")]
    Unknown,
}


