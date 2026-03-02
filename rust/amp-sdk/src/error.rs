use thiserror::Error;

pub type Result<T> = std::result::Result<T, AmpError>;

#[derive(Debug, Error)]
pub enum AmpError {
    #[error("invalid base URL: {0}")]
    InvalidUrl(String),

    #[error("missing API credentials: both api_key and hmac_secret are required for authenticated endpoints")]
    MissingCredentials,

    #[error("request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("HTTP {status}: {message}")]
    HttpStatus {
        status: u16,
        message: String,
        body: Option<String>,
    },

    #[error("state transition invalid: {0}")]
    InvalidStateTransition(String),
}
