//! UEFI vs. legacy BIOS detection.
//!
//! The presence of `/sys/firmware/efi` is the standard Linux signal that
//! the kernel was booted via UEFI; there is nothing to parse.

use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum BootMode {
    Uefi,
    Bios,
}

pub fn detect_boot_mode() -> BootMode {
    detect_boot_mode_at(Path::new("/sys/firmware/efi"))
}

pub fn detect_boot_mode_at(efi_dir: &Path) -> BootMode {
    if efi_dir.is_dir() {
        BootMode::Uefi
    } else {
        BootMode::Bios
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_uefi_when_efi_dir_exists() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir(tmp.path().join("efi")).unwrap();
        assert_eq!(detect_boot_mode_at(&tmp.path().join("efi")), BootMode::Uefi);
    }

    #[test]
    fn reports_bios_when_efi_dir_missing() {
        let tmp = tempfile::tempdir().unwrap();
        assert_eq!(
            detect_boot_mode_at(&tmp.path().join("nonexistent")),
            BootMode::Bios
        );
    }
}
