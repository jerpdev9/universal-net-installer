#[derive(Debug, thiserror::Error)]
pub enum InstallerError {
    #[error(transparent)]
    Core(#[from] uni_core::CoreError),

    #[error(transparent)]
    Storage(#[from] uni_storage::StorageError),

    #[error("no installer backend registered for id `{0}`")]
    UnknownBackend(String),

    #[error("validation failed: {0}")]
    ValidationFailed(String),

    #[error("`{0}` is not implemented yet")]
    NotImplemented(&'static str),
}

pub type Result<T> = std::result::Result<T, InstallerError>;
