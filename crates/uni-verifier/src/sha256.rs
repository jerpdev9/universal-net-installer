//! SHA-256 file hashing and verification.
//!
//! Universal Net Installer must never hand a downloaded artifact to an
//! installer without verifying it first (see `docs/security.md`); this is
//! the one integrity method implemented today.

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use sha2::{Digest, Sha256};

use crate::error::{Result, VerifierError};

const BUFFER_SIZE: usize = 1024 * 1024;

/// Streams `path` through SHA-256 and returns the lowercase hex digest.
/// Never loads the whole file into memory, so it's safe to use on
/// multi-gigabyte ISOs.
pub fn sha256_hex(path: &Path) -> Result<String> {
    let file = File::open(path).map_err(|source| uni_core::CoreError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; BUFFER_SIZE];

    loop {
        let read = reader
            .read(&mut buffer)
            .map_err(|source| uni_core::CoreError::Io {
                path: path.to_path_buf(),
                source,
            })?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }

    Ok(hex::encode(hasher.finalize()))
}

/// Verifies `path` against `expected_hex` (case-insensitive). Returns
/// `Ok(())` on a match, `Err(VerifierError::Mismatch)` otherwise.
pub fn verify_sha256(path: &Path, expected_hex: &str) -> Result<()> {
    let actual = sha256_hex(path)?;
    if actual.eq_ignore_ascii_case(expected_hex.trim()) {
        Ok(())
    } else {
        Err(VerifierError::Mismatch {
            path: path.display().to_string(),
            expected: expected_hex.trim().to_string(),
            actual,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn hashes_known_vector() {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        file.write_all(b"abc").unwrap();
        let digest = sha256_hex(file.path()).unwrap();
        // Well-known SHA-256("abc").
        assert_eq!(
            digest,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn verify_accepts_matching_checksum_case_insensitively() {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        file.write_all(b"abc").unwrap();
        let digest = sha256_hex(file.path()).unwrap();
        assert!(verify_sha256(file.path(), &digest.to_uppercase()).is_ok());
    }

    #[test]
    fn verify_rejects_mismatched_checksum() {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        file.write_all(b"abc").unwrap();
        let err = verify_sha256(
            file.path(),
            "0000000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap_err();
        assert!(matches!(err, VerifierError::Mismatch { .. }));
    }
}
