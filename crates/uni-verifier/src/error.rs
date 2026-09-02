#[derive(Debug, thiserror::Error)]
pub enum VerifierError {
    #[error(transparent)]
    Core(#[from] uni_core::CoreError),

    #[error("checksum mismatch for {path}: expected {expected}, got {actual}")]
    Mismatch {
        path: String,
        expected: String,
        actual: String,
    },

    #[error(
        "`{0}` is not implemented yet; the architecture is prepared but no code path executes it"
    )]
    NotImplemented(&'static str),
}

pub type Result<T> = std::result::Result<T, VerifierError>;
