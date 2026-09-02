//! RAM detection via `/proc/meminfo`.

use std::fs;
use std::path::Path;

use crate::error::{HardwareError, Result};

#[derive(Debug, Clone, serde::Serialize)]
pub struct MemoryInfo {
    pub total_bytes: u64,
}

pub fn detect_memory() -> Result<MemoryInfo> {
    detect_memory_at(Path::new("/proc/meminfo"))
}

pub fn detect_memory_at(path: &Path) -> Result<MemoryInfo> {
    let raw = fs::read_to_string(path).map_err(|source| {
        HardwareError::Core(uni_core::CoreError::Io {
            path: path.to_path_buf(),
            source,
        })
    })?;
    parse_meminfo(&raw)
}

/// Pure parser for `/proc/meminfo`. `MemTotal` is reported in kB by the
/// kernel; we normalize to bytes.
pub fn parse_meminfo(raw: &str) -> Result<MemoryInfo> {
    for line in raw.lines() {
        if let Some(rest) = line.strip_prefix("MemTotal:") {
            let kb_str = rest.trim().trim_end_matches("kB").trim();
            let kb: u64 = kb_str.parse().map_err(|_| HardwareError::Parse {
                what: "/proc/meminfo MemTotal".to_string(),
                reason: format!("`{kb_str}` is not a valid integer"),
            })?;
            return Ok(MemoryInfo {
                total_bytes: kb * 1024,
            });
        }
    }
    Err(HardwareError::Parse {
        what: "/proc/meminfo".to_string(),
        reason: "MemTotal line not found".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_mem_total() {
        let raw = "MemTotal:       32780000 kB\nMemFree:        10000000 kB\n";
        let mem = parse_meminfo(raw).unwrap();
        assert_eq!(mem.total_bytes, 32_780_000 * 1024);
    }

    #[test]
    fn errors_when_mem_total_missing() {
        assert!(parse_meminfo("MemFree: 100 kB\n").is_err());
    }
}
