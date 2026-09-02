//! Integrity verification for downloaded distribution artifacts.
//!
//! SHA-256 is fully implemented. SHA-512 and GPG signature verification
//! are modeled in [`VerificationMethod`] so the rest of the workspace can
//! already reference them, but no code path executes either yet — see
//! `docs/security.md`.

mod error;
mod sha256;
mod verifier;

pub use error::{Result, VerifierError};
pub use sha256::{sha256_hex, verify_sha256};
pub use verifier::{Sha256Verifier, VerificationMethod, Verifier};
