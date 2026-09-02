//! [`Verifier`]: the trait `uni-downloader` and `uni-installer` code
//! against. `Sha256Verifier` is the only implementation today; `Sha512`
//! and `Gpg` are modeled in [`VerificationMethod`] so callers and
//! manifests can already reference them, but dispatching to either
//! returns [`VerifierError::NotImplemented`] until a later phase.

use std::path::Path;

use crate::error::{Result, VerifierError};
use crate::sha256::verify_sha256;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VerificationMethod {
    Sha256,
    Sha512,
    Gpg,
}

pub trait Verifier {
    /// Verifies `path` against `expected` (checksum hex or signature,
    /// depending on `method`).
    fn verify(&self, path: &Path, method: VerificationMethod, expected: &str) -> Result<()>;
}

#[derive(Debug, Default)]
pub struct Sha256Verifier;

impl Verifier for Sha256Verifier {
    fn verify(&self, path: &Path, method: VerificationMethod, expected: &str) -> Result<()> {
        match method {
            VerificationMethod::Sha256 => verify_sha256(path, expected),
            VerificationMethod::Sha512 => {
                Err(VerifierError::NotImplemented("SHA-512 verification"))
            }
            VerificationMethod::Gpg => {
                Err(VerifierError::NotImplemented("GPG signature verification"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn dispatches_sha256_to_the_real_implementation() {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        file.write_all(b"abc").unwrap();
        let digest = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";

        let verifier = Sha256Verifier;
        assert!(
            verifier
                .verify(file.path(), VerificationMethod::Sha256, digest)
                .is_ok()
        );
    }

    #[test]
    fn sha512_and_gpg_are_prepared_but_not_implemented() {
        let file = tempfile::NamedTempFile::new().unwrap();
        let verifier = Sha256Verifier;

        assert!(matches!(
            verifier.verify(file.path(), VerificationMethod::Sha512, "x"),
            Err(VerifierError::NotImplemented(_))
        ));
        assert!(matches!(
            verifier.verify(file.path(), VerificationMethod::Gpg, "x"),
            Err(VerifierError::NotImplemented(_))
        ));
    }
}
