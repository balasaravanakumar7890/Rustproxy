use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Regex error: {0}")]
    RegexError(#[from] regex::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Invalid HTTP message: {0}")]
    InvalidHttpMessage(String),
}
