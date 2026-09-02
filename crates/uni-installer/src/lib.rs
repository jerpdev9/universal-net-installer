//! `InstallerBackend` trait and registry for launching official
//! distribution installers.
//!
//! No concrete backend (Ubuntu/Debian/Fedora/Arch) is implemented yet —
//! see roadmap phases 12-15 in `docs/roadmap.md`. This crate only fixes
//! the shape those backends will share and documents, in
//! `docs/boot-process.md`, which mechanism (kernel+initrd, netboot, kexec,
//! chainload, live installer) each distribution is expected to use.

mod backend;
mod error;

pub use backend::{BackendRegistry, InstallContext, InstallerBackend, ValidationReport};
pub use error::{InstallerError, Result};
