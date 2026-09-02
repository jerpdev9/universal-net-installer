#[derive(Debug, thiserror::Error)]
pub enum HardwareError {
    #[error(transparent)]
    Core(#[from] uni_core::CoreError),

    #[error(transparent)]
    Storage(#[from] uni_storage::StorageError),

    #[error(transparent)]
    Network(#[from] uni_network::NetworkError),

    #[error("failed to parse {what}: {reason}")]
    Parse { what: String, reason: String },
}

pub type Result<T> = std::result::Result<T, HardwareError>;
