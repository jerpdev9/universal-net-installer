#[derive(Debug, thiserror::Error)]
pub enum DownloaderError {
    #[error(transparent)]
    Core(#[from] uni_core::CoreError),

    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("downloaded file failed verification: {0}")]
    Verification(#[source] uni_verifier::VerifierError),

    #[error("download cancelled")]
    Cancelled,

    #[error("no mirror in the request succeeded")]
    AllMirrorsFailed,

    #[error("request has no mirrors to try")]
    NoMirrors,
}

pub type Result<T> = std::result::Result<T, DownloaderError>;
