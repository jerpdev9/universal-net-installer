//! Resume support: how much of `destination` already exists on disk.

use std::path::Path;

/// Bytes already present at `destination`, i.e. the offset a resumed
/// download should request via an HTTP `Range` header. `0` if the file
/// does not exist yet.
pub fn resume_offset(destination: &Path) -> u64 {
    std::fs::metadata(destination).map(|m| m.len()).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn returns_zero_for_a_missing_file() {
        let tmp = tempfile::tempdir().unwrap();
        assert_eq!(resume_offset(&tmp.path().join("does-not-exist")), 0);
    }

    #[test]
    fn returns_existing_file_length() {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        file.write_all(b"0123456789").unwrap();
        assert_eq!(resume_offset(file.path()), 10);
    }
}
