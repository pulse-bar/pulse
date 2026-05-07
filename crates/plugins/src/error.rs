use thiserror::Error;

#[derive(Debug, Error)]
pub enum PluginError {
    #[error("not configured: {0}")]
    NotConfigured(String),

    #[error("auth: {0}")]
    Auth(String),

    #[error("transport: {0}")]
    Transport(String),

    #[error("instance not found: {0}")]
    InstanceNotFound(String),

    #[error("{0}")]
    Other(String),
}

pub type PluginResult<T> = Result<T, PluginError>;
