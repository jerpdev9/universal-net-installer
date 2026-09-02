//! Identifies the disk Universal Net Installer itself booted from, so it
//! can be marked [`ProtectionState::Protected`][crate::disk::ProtectionState]
//! and excluded from any future destructive operation.
//!
//! Debian Live (`live-boot`) exposes the medium at `/run/live/medium`; a
//! plain installed system exposes its root at `/`. We try the live path
//! first and fall back to `/`, since during phase-1 development this often
//! runs outside a live environment.

use crate::disk::{DiskInfo, ProtectionState};

const LIVE_MEDIUM_MOUNTPOINT: &str = "/run/live/medium";
const ROOT_MOUNTPOINT: &str = "/";

/// Resolves the boot source device and flips the matching [`DiskInfo`] to
/// `Protected`. Silently does nothing if the source can't be resolved
/// (e.g. running in a container during development) rather than guessing.
pub fn mark_boot_device(disks: &mut [DiskInfo]) {
    let Some(disk_name) = detect_boot_disk_name() else {
        return;
    };
    if let Some(disk) = disks.iter_mut().find(|d| d.name == disk_name) {
        disk.protection = ProtectionState::Protected;
    }
}

fn detect_boot_disk_name() -> Option<String> {
    let source = uni_core::process::run("findmnt", &["-no", "SOURCE", LIVE_MEDIUM_MOUNTPOINT])
        .ok()
        .filter(|s| !s.is_empty())
        .or_else(|| {
            uni_core::process::run("findmnt", &["-no", "SOURCE", ROOT_MOUNTPOINT])
                .ok()
                .filter(|s| !s.is_empty())
        })?;
    parse_boot_source(&source)
}

/// Pure helper: turns a `findmnt SOURCE` value (e.g. `/dev/sdb1`) into the
/// parent disk name (e.g. `sdb`). Returns `None` for sources that are not a
/// real block device (`overlay`, `tmpfs`, ...).
pub fn parse_boot_source(source: &str) -> Option<String> {
    let source = source.trim();
    if !source.starts_with("/dev/") {
        return None;
    }
    Some(partition_to_disk_name(source))
}

/// Strips a partition suffix off a device path, returning the parent disk
/// name: `/dev/sda1` -> `sda`, `/dev/nvme0n1p1` -> `nvme0n1`,
/// `/dev/mmcblk0p1` -> `mmcblk0`.
pub fn partition_to_disk_name(partition_path: &str) -> String {
    let name = partition_path.rsplit('/').next().unwrap_or(partition_path);

    if let Some(p_idx) = name.rfind('p') {
        let (head, tail) = name.split_at(p_idx);
        let digits = &tail[1..];
        let head_ends_in_digit = head.ends_with(|c: char| c.is_ascii_digit());
        if !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit()) && head_ends_in_digit {
            return head.to_string();
        }
    }

    name.trim_end_matches(|c: char| c.is_ascii_digit())
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_simple_partition_suffix() {
        assert_eq!(partition_to_disk_name("/dev/sda1"), "sda");
        assert_eq!(partition_to_disk_name("/dev/sdb12"), "sdb");
    }

    #[test]
    fn strips_nvme_and_mmc_partition_suffix() {
        assert_eq!(partition_to_disk_name("/dev/nvme0n1p1"), "nvme0n1");
        assert_eq!(partition_to_disk_name("/dev/mmcblk0p1"), "mmcblk0");
    }

    #[test]
    fn parse_boot_source_rejects_non_device_sources() {
        assert_eq!(parse_boot_source("overlay"), None);
        assert_eq!(parse_boot_source("tmpfs"), None);
        assert_eq!(parse_boot_source(""), None);
    }

    #[test]
    fn parse_boot_source_accepts_device_paths() {
        assert_eq!(parse_boot_source("/dev/sdb1\n"), Some("sdb".to_string()));
    }
}
