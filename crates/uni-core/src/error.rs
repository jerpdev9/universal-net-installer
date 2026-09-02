use std::path::PathBuf;

/// Errors shared by every Universal Net Installer crate.
///
/// Domain crates (`uni-hardware`, `uni-storage`, ...) define their own
/// error enums for domain-specific failures and wrap [`CoreError`] via
/// `#[from]` or `#[source]` when a lower-level primitive fails.
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("io error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("command `{command}` failed with status {status}: {stderr}")]
    CommandFailed {
        command: String,
        status: i32,
        stderr: String,
    },

    #[error("command `{command}` could not be executed: {source}")]
    CommandNotRunnable {
        command: String,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to parse {what}: {reason}")]
    Parse { what: String, reason: String },

    #[error("{0} is not implemented yet")]
    NotImplemented(&'static str),
}

pub type Result<T> = std::result::Result<T, CoreError>;
