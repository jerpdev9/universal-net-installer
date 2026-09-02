//! Resumable, mirror-aware HTTPS downloader with progress reporting and
//! cancellation.
//!
//! [`Downloader::download_and_verify_sha256`] is the intended entry point:
//! it composes [`Downloader::download`] with `uni-verifier` so a caller
//! cannot end up with an unverified file marked as ready to use. Metalink
//! and torrent sources are designed for but not implemented yet.

mod downloader;
mod error;
mod mirror;
mod progress;
mod resume;

pub use downloader::{DownloadOutcome, DownloadRequest, Downloader};
pub use error::{DownloaderError, Result};
pub use mirror::next_mirror;
pub use progress::{CancellationToken, NullProgressSink, Progress, ProgressSink};
pub use resume::resume_offset;
