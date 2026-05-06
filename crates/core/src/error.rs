use thiserror::Error;

#[derive(Debug, Error)]
pub enum PulseError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("connection pool: {0}")]
    Pool(#[from] r2d2::Error),

    #[error("json: {0}")]
    Json(#[from] serde_json::Error),

    #[error("{0}")]
    Other(String),
}

impl From<anyhow::Error> for PulseError {
    fn from(value: anyhow::Error) -> Self {
        PulseError::Other(format!("{value:#}"))
    }
}

pub type PulseResult<T> = Result<T, PulseError>;
