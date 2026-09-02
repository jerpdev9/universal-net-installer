//! [`Downloader`]: HTTPS download with mirror fallback, resume, progress
//! reporting and cancellation.
//!
//! Nothing in this crate calls `uni-verifier` automatically after a
//! download completes — wiring "never install an unverified artifact" is
//! the caller's job (`uni-installer`, in a later phase), by design: this
//! type's job is only to get bytes onto disk correctly. See
//! `docs/security.md` for the "never skip verification" rule this
//! supports.
//!
//! Metalink and torrent sources are out of scope for the MVP; the mirror
//! list on [`DownloadRequest`] is deliberately just a `Vec<String>` of
//! HTTPS URLs so adding another `SourceKind` later doesn't require
//! reshaping this API.

use std::path::PathBuf;
use std::time::Duration;

use tokio::io::AsyncWriteExt;

use crate::error::{DownloaderError, Result};
use crate::mirror::next_mirror;
use crate::progress::{CancellationToken, Progress, ProgressSink};
use crate::resume::resume_offset;

#[derive(Debug, Clone)]
pub struct DownloadRequest {
    /// Candidate URLs in priority order; the same resource on different
    /// mirrors.
    pub mirrors: Vec<String>,
    pub destination: PathBuf,
    /// Used as the progress total when the server omits `Content-Length`.
    pub expected_size: Option<u64>,
    pub timeout: Duration,
}

#[derive(Debug, Clone, Copy)]
pub struct DownloadOutcome {
    pub bytes_written: u64,
}

pub struct Downloader {
    client: reqwest::Client,
}

impl Default for Downloader {
    fn default() -> Self {
        Self::new()
    }
}

impl Downloader {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    /// Downloads `request`, trying each mirror in order until one
    /// succeeds. A partially-downloaded `destination` from a previous
    /// attempt is resumed via an HTTP `Range` request rather than
    /// restarted.
    pub async fn download(
        &self,
        request: &DownloadRequest,
        sink: &dyn ProgressSink,
        cancel: &CancellationToken,
    ) -> Result<DownloadOutcome> {
        if request.mirrors.is_empty() {
            return Err(DownloaderError::NoMirrors);
        }

        let mut attempted: Vec<&str> = Vec::new();
        let mut last_error: Option<DownloaderError> = None;

        while let Some(mirror) = next_mirror(&request.mirrors, &attempted) {
            attempted.push(mirror);
            match self.try_download(mirror, request, sink, cancel).await {
                Ok(outcome) => return Ok(outcome),
                Err(DownloaderError::Cancelled) => return Err(DownloaderError::Cancelled),
                Err(err) => {
                    tracing::warn!(mirror, %err, "mirror failed, trying next");
                    last_error = Some(err);
                }
            }
        }

        Err(last_error.unwrap_or(DownloaderError::AllMirrorsFailed))
    }

    async fn try_download(
        &self,
        url: &str,
        request: &DownloadRequest,
        sink: &dyn ProgressSink,
        cancel: &CancellationToken,
    ) -> Result<DownloadOutcome> {
        let offset = resume_offset(&request.destination);

        let mut http_request = self.client.get(url).timeout(request.timeout);
        if offset > 0 {
            http_request = http_request.header(reqwest::header::RANGE, format!("bytes={offset}-"));
        }

        let mut response = http_request.send().await?.error_for_status()?;
        let total = response
            .content_length()
            .map(|len| len + offset)
            .or(request.expected_size);

        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(offset > 0)
            .truncate(offset == 0)
            .open(&request.destination)
            .await
            .map_err(|source| uni_core::CoreError::Io {
                path: request.destination.clone(),
                source,
            })?;

        let mut downloaded = offset;
        while let Some(chunk) = response.chunk().await? {
            if cancel.is_cancelled() {
                return Err(DownloaderError::Cancelled);
            }
            file.write_all(&chunk)
                .await
                .map_err(|source| uni_core::CoreError::Io {
                    path: request.destination.clone(),
                    source,
                })?;
            downloaded += chunk.len() as u64;
            sink.on_progress(Progress { downloaded, total });
        }

        Ok(DownloadOutcome {
            bytes_written: downloaded,
        })
    }

    /// Downloads `request`, then verifies the result against
    /// `expected_sha256` before returning. The verification failure and
    /// the download failure share this crate's [`Result`], so a caller
    /// cannot accidentally treat an unverified file as usable — the only
    /// `Ok` path here is "downloaded and verified".
    pub async fn download_and_verify_sha256(
        &self,
        request: &DownloadRequest,
        expected_sha256: &str,
        sink: &dyn ProgressSink,
        cancel: &CancellationToken,
    ) -> Result<DownloadOutcome> {
        let outcome = self.download(request, sink, cancel).await?;
        uni_verifier::verify_sha256(&request.destination, expected_sha256)
            .map_err(DownloaderError::Verification)?;
        Ok(outcome)
    }
}
