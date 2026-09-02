//! `StorageGuard`: the single, mandatory gate in front of any destructive
//! disk operation.
//!
//! Phase 1 deliberately implements only the *validation* and *warning
//! rendering* half of this API. There is no `execute()` method and no code
//! path anywhere in this crate that invokes `dd`, `wipefs`, `mkfs`,
//! `fdisk`, `parted` or `sgdisk`. That capability is intentionally deferred
//! to a later phase, where it will be re-confirmed and documented in
//! `docs/security.md` before it lands.

use crate::disk::{DiskInfo, format_size};
use crate::error::{Result, StorageError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DestructiveAction {
    EraseDisk,
    CreatePartitionTable,
    FormatPartition,
    WriteRawImage,
}

impl DestructiveAction {
    pub fn label(self) -> &'static str {
        match self {
            DestructiveAction::EraseDisk => "ERASE ENTIRE DISK",
            DestructiveAction::CreatePartitionTable => "CREATE NEW PARTITION TABLE",
            DestructiveAction::FormatPartition => "FORMAT PARTITION",
            DestructiveAction::WriteRawImage => "WRITE RAW IMAGE",
        }
    }
}

#[derive(Debug, Clone)]
pub struct DestructiveRequest {
    pub action: DestructiveAction,
    /// Disk path or name this request targets, e.g. `/dev/nvme0n1`.
    pub target: String,
}

/// Gate in front of every destructive storage operation.
#[derive(Debug, Default)]
pub struct StorageGuard;

impl StorageGuard {
    pub fn new() -> Self {
        Self
    }

    /// Checks that `request.target` exists in `disks` and is not
    /// [`Protected`][crate::disk::ProtectionState::Protected]. Read-only:
    /// performs no I/O against the device itself.
    pub fn validate(&self, disks: &[DiskInfo], request: &DestructiveRequest) -> Result<()> {
        let disk = disks
            .iter()
            .find(|d| d.path == request.target || d.name == request.target)
            .ok_or_else(|| StorageError::DeviceNotFound(request.target.clone()))?;

        if disk.is_protected() {
            return Err(StorageError::ProtectedDevice(disk.path.clone()));
        }

        Ok(())
    }

    /// Renders the mandatory confirmation prompt (device, model, serial,
    /// size, partitions, action) the UI must show before a user can
    /// confirm a destructive action.
    pub fn confirmation_prompt(&self, disk: &DiskInfo, request: &DestructiveRequest) -> String {
        let partitions = if disk.partitions.is_empty() {
            "(none)".to_string()
        } else {
            disk.partitions
                .iter()
                .map(|p| p.path.clone())
                .collect::<Vec<_>>()
                .join(", ")
        };

        format!(
            "WARNING\n\nDevice:\n{}\n\nModel:\n{}\n\nSerial:\n{}\n\nSize:\n{}\n\nPartitions:\n{}\n\nAction:\n{}",
            disk.path,
            disk.model.as_deref().unwrap_or("unknown"),
            disk.serial.as_deref().unwrap_or("unknown"),
            format_size(disk.size_bytes),
            partitions,
            request.action.label(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::disk::{DiskKind, PartitionInfo, ProtectionState};

    fn disk(name: &str, protection: ProtectionState) -> DiskInfo {
        DiskInfo {
            name: name.to_string(),
            path: format!("/dev/{name}"),
            kind: DiskKind::Nvme,
            size_bytes: 2_000_398_934_016,
            model: Some("Samsung 990 Pro".to_string()),
            serial: Some("S6XPNS0R999999".to_string()),
            removable: false,
            partitions: vec![PartitionInfo {
                name: format!("{name}1"),
                path: format!("/dev/{name}1"),
                size_bytes: 2_000_000_000_000,
                fstype: Some("ext4".to_string()),
                mountpoint: None,
            }],
            protection,
        }
    }

    #[test]
    fn validate_accepts_unprotected_known_device() {
        let disks = vec![disk("nvme0n1", ProtectionState::Normal)];
        let req = DestructiveRequest {
            action: DestructiveAction::EraseDisk,
            target: "/dev/nvme0n1".to_string(),
        };
        assert!(StorageGuard::new().validate(&disks, &req).is_ok());
    }

    #[test]
    fn validate_rejects_protected_device() {
        let disks = vec![disk("sdb", ProtectionState::Protected)];
        let req = DestructiveRequest {
            action: DestructiveAction::EraseDisk,
            target: "sdb".to_string(),
        };
        let err = StorageGuard::new().validate(&disks, &req).unwrap_err();
        assert!(matches!(err, StorageError::ProtectedDevice(_)));
    }

    #[test]
    fn validate_rejects_unknown_device() {
        let disks: Vec<DiskInfo> = vec![];
        let req = DestructiveRequest {
            action: DestructiveAction::EraseDisk,
            target: "/dev/sdz".to_string(),
        };
        let err = StorageGuard::new().validate(&disks, &req).unwrap_err();
        assert!(matches!(err, StorageError::DeviceNotFound(_)));
    }

    #[test]
    fn confirmation_prompt_contains_all_mandatory_fields() {
        let d = disk("nvme0n1", ProtectionState::Normal);
        let req = DestructiveRequest {
            action: DestructiveAction::EraseDisk,
            target: d.path.clone(),
        };
        let prompt = StorageGuard::new().confirmation_prompt(&d, &req);
        for expected in [
            "WARNING",
            "/dev/nvme0n1",
            "Samsung 990 Pro",
            "S6XPNS0R999999",
            "1.8 TB",
            "/dev/nvme0n11",
            "ERASE ENTIRE DISK",
        ] {
            assert!(
                prompt.contains(expected),
                "prompt missing {expected:?}: {prompt}"
            );
        }
    }
}
