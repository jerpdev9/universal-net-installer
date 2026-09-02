//! Progress reporting and cancellation, shared across the mirror-retry
//! loop in [`crate::Downloader::download`].

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug, Clone, Copy)]
pub struct Progress {
    pub downloaded: u64,
    /// `None` when the server did not report `Content-Length` and no
    /// `expected_size` was set on the request.
    pub total: Option<u64>,
}

pub trait ProgressSink: Send + Sync {
    fn on_progress(&self, progress: Progress);
}

/// Discards progress updates. Useful for callers (tests, batch scripts)
/// that don't render a UI.
#[derive(Debug, Default)]
pub struct NullProgressSink;

impl ProgressSink for NullProgressSink {
    fn on_progress(&self, _progress: Progress) {}
}

/// Cooperative cancellation flag shared between the UI/caller and an
/// in-flight download.
#[derive(Debug, Clone, Default)]
pub struct CancellationToken(Arc<AtomicBool>);

impl CancellationToken {
    pub fn new() -> Self {
        Self(Arc::new(AtomicBool::new(false)))
    }

    pub fn cancel(&self) {
        self.0.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancellation_token_starts_uncancelled_and_latches() {
        let token = CancellationToken::new();
        assert!(!token.is_cancelled());
        token.cancel();
        assert!(token.is_cancelled());
    }

    #[test]
    fn cancellation_token_clones_share_state() {
        let token = CancellationToken::new();
        let clone = token.clone();
        clone.cancel();
        assert!(token.is_cancelled());
    }
}
