//! Loads and validates the YAML distribution manifests under
//! `manifests/`. No distribution or version is hardcoded here — see
//! `docs/manifests.md`.

mod error;
mod loader;
mod manifest;

pub use error::{CatalogError, Result};
pub use loader::{load_catalog_dir, load_from_path, load_from_str};
pub use manifest::{InstallerRef, Manifest, Release, Source, SourceKind, Verification};
