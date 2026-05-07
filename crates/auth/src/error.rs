use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("not configured: {0}")]
    NotConfigured(String),

    #[error("keychain: {0}")]
    Keychain(String),

    #[error("oauth: {0}")]
    OAuth(String),

    #[error("transport: {0}")]
    Transport(String),

    #[error("token expired and could not be refreshed")]
    TokenExpired,

    #[error("flow {0} not found or already completed")]
    UnknownFlow(String),

    #[error("{0}")]
    Other(String),
}

pub type AuthResult<T> = Result<T, AuthError>;
