//! Logging initialization shared by every binary in the workspace.
//!
//! The TUI owns the terminal, so logs must never go to stdout/stderr while
//! it is running; callers pass a [`std::io::Write`] writer (typically a
//! rolling file appender) instead.

use tracing_subscriber::EnvFilter;

/// Initializes a global `tracing` subscriber writing to `writer`.
///
/// The filter defaults to `info` and honors `RUST_LOG` when set. Returns an
/// error only if a global subscriber was already installed.
pub fn init<W>(writer: W) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    W: for<'a> tracing_subscriber::fmt::MakeWriter<'a> + Send + Sync + 'static,
{
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(writer)
        .with_ansi(false)
        .try_init()
}
