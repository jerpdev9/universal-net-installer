#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error(transparent)]
    Core(#[from] uni_core::CoreError),

    #[error("failed to parse lsblk output: {0}")]
    Parse(String),

    #[error("device `{0}` not found in the current disk inventory")]
    DeviceNotFound(String),

    #[error(
        "refusing to operate on `{0}`: this device is PROTECTED (it is the Universal Net Installer boot medium)"
    )]
    ProtectedDevice(String),
}

pub type Result<T> = std::result::Result<T, StorageError>;
