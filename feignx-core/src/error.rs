#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("transport error: {0}")]
    Transport(Box<dyn std::error::Error + Send + Sync>),

    #[error("HTTP {status}: {body}")]
    Status { status: u16, body: String },

    #[error("{code}: {message}")]
    Business { code: String, message: String },

    #[error("decode error: {0}")]
    Decode(#[from] serde_json::Error),

    #[error("service resolve failed: {0}")]
    Resolve(String),

    #[error("circuit breaker open")]
    CircuitOpen,

    #[error("request cancelled")]
    Cancelled,

    #[error("request timeout")]
    Timeout,

    #[error("encode error: {0}")]
    Encode(String),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, Error>;