#[derive(Debug, thiserror::Error)]
pub enum NetworkError {
    #[error(transparent)]
    Core(#[from] uni_core::CoreError),

    #[error("interface `{0}` not found")]
    InterfaceNotFound(String),

    #[error("failed to parse {what}: {reason}")]
    Parse { what: String, reason: String },
}

pub type Result<T> = std::result::Result<T, NetworkError>;
