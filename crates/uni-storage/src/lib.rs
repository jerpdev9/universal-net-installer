//! Disk/partition detection, boot-device identification and the
//! destructive-operation gate ([`StorageGuard`]).
//!
//! See `docs/storage-safety.md` for the safety model this crate
//! implements: nothing here can erase, format or write to a disk. That
//! capability is added later, behind `StorageGuard`, with its own review.

mod boot_device;
mod disk;
mod error;
mod guard;

pub use boot_device::{parse_boot_source, partition_to_disk_name};
pub use disk::{
    DiskInfo, DiskKind, PartitionInfo, ProtectionState, detect_disks, format_size, parse_lsblk_json,
};
pub use error::{Result, StorageError};
pub use guard::{DestructiveAction, DestructiveRequest, StorageGuard};
