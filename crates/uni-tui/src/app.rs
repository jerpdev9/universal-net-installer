//! Application state. Deliberately holds only what phase 1 renders: a
//! [`uni_hardware::HardwareSnapshot`] and refresh/error status. No OS
//! selection, download or install state exists yet — that lands with the
//! catalog/downloader/installer phases in `docs/roadmap.md`.

use std::time::Instant;

use uni_hardware::HardwareSnapshot;

pub struct App {
    pub snapshot: Option<HardwareSnapshot>,
    pub error: Option<String>,
    pub last_refreshed: Option<Instant>,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            snapshot: None,
            error: None,
            last_refreshed: None,
            should_quit: false,
        }
    }

    /// Re-runs hardware detection. Detection is a handful of fast reads
    /// (`/proc`, `/sys`) and short-lived commands (`lsblk`, `lspci`), so
    /// running it synchronously on key-press keeps the TUI simple without
    /// a noticeable stall.
    pub fn refresh(&mut self) {
        match uni_hardware::detect() {
            Ok(snapshot) => {
                self.snapshot = Some(snapshot);
                self.error = None;
            }
            Err(err) => {
                self.error = Some(err.to_string());
            }
        }
        self.last_refreshed = Some(Instant::now());
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
