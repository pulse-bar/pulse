use thiserror::Error;

#[derive(Debug, Error)]
pub enum EnrichmentError {
    #[error("not configured: {0}")]
    NotConfigured(String),

    #[error("auth: {0}")]
    Auth(String),

    #[error("transport: {0}")]
    Transport(String),

    #[error("parse: {0}")]
    Parse(String),

    #[error("rate limited; retry after {retry_after_secs}s")]
    RateLimited { retry_after_secs: u64 },

    #[error("{0}")]
    Other(String),
}

pub type EnrichmentResult<T> = Result<T, EnrichmentError>;
