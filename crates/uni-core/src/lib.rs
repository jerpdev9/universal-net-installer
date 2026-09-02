//! Foundation crate for Universal Net Installer.
//!
//! `uni-core` holds nothing domain-specific: shared error types, the
//! instrumented process-execution wrapper every "shells out to a system
//! tool" crate builds on, logging setup, and the [`arch::Architecture`]
//! enum shared between hardware detection and catalog manifests. See
//! `docs/architecture.md` for how this fits into the rest of the
//! workspace.

pub mod arch;
pub mod error;
pub mod logging;
pub mod process;

pub use arch::Architecture;
pub use error::{CoreError, Result};
