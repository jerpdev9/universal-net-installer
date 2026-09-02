#[derive(Debug, thiserror::Error)]
pub enum CatalogError {
    #[error(transparent)]
    Core(#[from] uni_core::CoreError),

    #[error("failed to parse manifest {path}: {reason}")]
    Parse { path: String, reason: String },

    #[error("manifest {path} has no releases")]
    EmptyReleases { path: String },
}

pub type Result<T> = std::result::Result<T, CatalogError>;
