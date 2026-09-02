//! Disk inventory via `lsblk -J -b -O`.
//!
//! We parse JSON rather than the plain-text `lsblk` table: the JSON schema
//! is stable across distributions and column widths, so this is the
//! "better API" the project's parsing guidance asks us to prefer.

use serde_json::Value;

use crate::error::{Result, StorageError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum DiskKind {
    Hdd,
    SataSsd,
    Nvme,
    Usb,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum ProtectionState {
    /// This is the medium Universal Net Installer booted from. It must
    /// never be offered as an install/erase target.
    Protected,
    Normal,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PartitionInfo {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub fstype: Option<String>,
    pub mountpoint: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DiskInfo {
    pub name: String,
    pub path: String,
    pub kind: DiskKind,
    pub size_bytes: u64,
    pub model: Option<String>,
    pub serial: Option<String>,
    pub removable: bool,
    pub partitions: Vec<PartitionInfo>,
    pub protection: ProtectionState,
}

impl DiskInfo {
    pub fn is_protected(&self) -> bool {
        matches!(self.protection, ProtectionState::Protected)
    }
}

/// Detects all disks (not partitions) currently visible to the kernel and
/// marks the boot medium as [`ProtectionState::Protected`].
///
/// This function only reads state (`lsblk`, `findmnt`, `/proc`); it never
/// modifies anything on disk.
pub fn detect_disks() -> Result<Vec<DiskInfo>> {
    let raw = uni_core::process::run("lsblk", &["-J", "-b", "-O"])?;
    let mut disks = parse_lsblk_json(&raw)?;
    crate::boot_device::mark_boot_device(&mut disks);
    Ok(disks)
}

/// Pure parser: turns raw `lsblk -J -b -O` JSON into [`DiskInfo`] entries.
/// Kept separate from [`detect_disks`] so it can be exercised with fixture
/// data in tests without shelling out.
pub fn parse_lsblk_json(raw: &str) -> Result<Vec<DiskInfo>> {
    let root: Value = serde_json::from_str(raw).map_err(|e| StorageError::Parse(e.to_string()))?;
    let devices = root
        .get("blockdevices")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    Ok(devices
        .iter()
        .filter(|dev| dev.get("type").and_then(Value::as_str) == Some("disk"))
        .map(parse_disk)
        .collect())
}

fn as_string(v: &Value, field: &str) -> Option<String> {
    match v.get(field) {
        None | Some(Value::Null) => None,
        Some(Value::String(s)) => Some(s.clone()),
        Some(other) => Some(other.to_string()),
    }
}

fn as_u64(v: &Value, field: &str) -> u64 {
    match v.get(field) {
        Some(Value::Number(n)) => n.as_u64().unwrap_or(0),
        Some(Value::String(s)) => s.parse().unwrap_or(0),
        _ => 0,
    }
}

fn as_bool_flag(v: &Value, field: &str) -> bool {
    match v.get(field) {
        Some(Value::Bool(b)) => *b,
        Some(Value::String(s)) => s == "1",
        Some(Value::Number(n)) => n.as_u64() == Some(1),
        _ => false,
    }
}

fn classify(tran: Option<&str>, rotational: bool, removable: bool) -> DiskKind {
    let base = match tran {
        Some("nvme") => DiskKind::Nvme,
        Some("usb") => DiskKind::Usb,
        Some("sata") | Some("ata") | Some("sas") | Some("scsi") if rotational => DiskKind::Hdd,
        Some("sata") | Some("ata") | Some("sas") | Some("scsi") => DiskKind::SataSsd,
        _ => DiskKind::Unknown,
    };
    if removable && base != DiskKind::Nvme {
        DiskKind::Usb
    } else {
        base
    }
}

fn parse_disk(dev: &Value) -> DiskInfo {
    let name = as_string(dev, "name").unwrap_or_default();
    let path = as_string(dev, "path").unwrap_or_else(|| format!("/dev/{name}"));
    let tran = as_string(dev, "tran");
    let rotational = as_bool_flag(dev, "rota");
    let removable = as_bool_flag(dev, "rm");

    let partitions = dev
        .get("children")
        .and_then(Value::as_array)
        .map(|children| {
            children
                .iter()
                .filter(|c| c.get("type").and_then(Value::as_str) == Some("part"))
                .map(|c| PartitionInfo {
                    name: as_string(c, "name").unwrap_or_default(),
                    path: as_string(c, "path").unwrap_or_default(),
                    size_bytes: as_u64(c, "size"),
                    fstype: as_string(c, "fstype"),
                    mountpoint: as_string(c, "mountpoint"),
                })
                .collect()
        })
        .unwrap_or_default();

    DiskInfo {
        kind: classify(tran.as_deref(), rotational, removable),
        size_bytes: as_u64(dev, "size"),
        model: as_string(dev, "model"),
        serial: as_string(dev, "serial"),
        removable,
        partitions,
        protection: ProtectionState::Normal,
        name,
        path,
    }
}

/// Formats a byte count as a human-readable size (`2.0 TB`, `500.0 GB`, ...).
pub fn format_size(bytes: u64) -> String {
    const UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];
    let mut size = bytes as f64;
    let mut unit = 0;
    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }
    format!("{size:.1} {}", UNITS[unit])
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
    {
       "blockdevices": [
          {"name":"sda","path":"/dev/sda","size":500107862016,"type":"disk","tran":"sata","rota":"0","rm":"0","model":"Samsung SSD 870","serial":"S6P1NS0R123456","children":[
             {"name":"sda1","path":"/dev/sda1","size":536870912,"type":"part","fstype":"vfat","mountpoint":"/boot/efi"},
             {"name":"sda2","path":"/dev/sda2","size":499569655808,"type":"part","fstype":"ext4","mountpoint":"/"}
          ]},
          {"name":"nvme0n1","path":"/dev/nvme0n1","size":2000398934016,"type":"disk","tran":"nvme","rota":"0","rm":"0","model":"Samsung 990 Pro","serial":"S6XPNS0R999999","children":[]},
          {"name":"sdb","path":"/dev/sdb","size":16008609792,"type":"disk","tran":"usb","rota":"0","rm":"1","model":"Kingston DataTraveler","serial":"0013728912AB","children":[
             {"name":"sdb1","path":"/dev/sdb1","size":16000000000,"type":"part","fstype":"vfat","mountpoint":"/run/live/medium"}
          ]}
       ]
    }
    "#;

    #[test]
    fn parses_disks_and_partitions() {
        let disks = parse_lsblk_json(SAMPLE).unwrap();
        assert_eq!(disks.len(), 3);

        let sda = disks.iter().find(|d| d.name == "sda").unwrap();
        assert_eq!(sda.kind, DiskKind::SataSsd);
        assert_eq!(sda.size_bytes, 500_107_862_016);
        assert_eq!(sda.partitions.len(), 2);
        assert_eq!(sda.model.as_deref(), Some("Samsung SSD 870"));
    }

    #[test]
    fn classifies_nvme_and_usb() {
        let disks = parse_lsblk_json(SAMPLE).unwrap();
        let nvme = disks.iter().find(|d| d.name == "nvme0n1").unwrap();
        assert_eq!(nvme.kind, DiskKind::Nvme);

        let usb = disks.iter().find(|d| d.name == "sdb").unwrap();
        assert_eq!(usb.kind, DiskKind::Usb);
        assert!(usb.removable);
    }

    #[test]
    fn formats_human_readable_sizes() {
        assert_eq!(format_size(500_107_862_016), "465.8 GB");
        assert_eq!(format_size(2_000_398_934_016), "1.8 TB");
        assert_eq!(format_size(0), "0.0 B");
    }
}
