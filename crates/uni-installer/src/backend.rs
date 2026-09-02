//! [`InstallerBackend`]: the trait a per-distribution installer (Ubuntu,
//! Debian, Fedora, Arch, ...) implements. No concrete backend exists yet —
//! that's Fase 12-15 of the roadmap; this phase only fixes the shape they
//! will share, and provides the [`BackendRegistry`] that will look them up
//! by the `installer.backend` id from a catalog manifest.
//!
//! `launch()` is documented as never being reached without a prior
//! successful `validate()` and integrity-verified source: see
//! `docs/security.md` and `docs/boot-process.md` for the
//! kernel+initrd/netboot/kexec/chainload mechanisms concrete backends will
//! use.

use std::path::PathBuf;

use crate::error::{InstallerError, Result};

/// Everything a backend needs to validate, prepare and launch an install.
#[derive(Debug, Clone)]
pub struct InstallContext {
    pub manifest_id: String,
    pub release_version: String,
    /// Verified (SHA-256/GPG-checked) local path to the boot resource
    /// (ISO, or kernel+initrd pair) this backend consumes.
    pub source_path: PathBuf,
    /// Target device for the install, if already chosen. Backends must
    /// treat this as opaque and never act on it directly — all destructive
    /// operations go through `uni_storage::StorageGuard`.
    pub target_device: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ValidationReport {
    pub ok: bool,
    pub messages: Vec<String>,
}

pub trait InstallerBackend {
    /// Stable id matched against `installer.backend` in a catalog
    /// manifest (e.g. `"ubuntu"`, `"debian"`, `"fedora"`, `"arch"`).
    fn id(&self) -> &'static str;

    /// Checks that `ctx` is internally consistent and its source is
    /// usable (e.g. the ISO mounts, the kernel/initrd pair matches).
    /// Must not touch any disk device.
    fn validate(&self, ctx: &InstallContext) -> Result<ValidationReport>;

    /// Stages whatever the launch step needs (e.g. copying kernel/initrd
    /// to an accessible location, writing a boot entry). Must not perform
    /// any operation `StorageGuard` would classify as destructive without
    /// having gone through it first.
    fn prepare(&self, ctx: &InstallContext) -> Result<()>;

    /// Hands off to the distribution's own official installer.
    fn launch(&self, ctx: &InstallContext) -> Result<()>;
}

/// Looks up an [`InstallerBackend`] by its [`InstallerBackend::id`].
#[derive(Default)]
pub struct BackendRegistry {
    backends: Vec<Box<dyn InstallerBackend>>,
}

impl BackendRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, backend: Box<dyn InstallerBackend>) {
        self.backends.push(backend);
    }

    pub fn get(&self, id: &str) -> Result<&dyn InstallerBackend> {
        self.backends
            .iter()
            .find(|b| b.id() == id)
            .map(|b| b.as_ref())
            .ok_or_else(|| InstallerError::UnknownBackend(id.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockBackend;

    impl InstallerBackend for MockBackend {
        fn id(&self) -> &'static str {
            "mock"
        }

        fn validate(&self, _ctx: &InstallContext) -> Result<ValidationReport> {
            Ok(ValidationReport {
                ok: true,
                messages: vec!["mock backend always validates".to_string()],
            })
        }

        fn prepare(&self, _ctx: &InstallContext) -> Result<()> {
            Ok(())
        }

        fn launch(&self, _ctx: &InstallContext) -> Result<()> {
            Err(InstallerError::NotImplemented("mock launch"))
        }
    }

    fn ctx() -> InstallContext {
        InstallContext {
            manifest_id: "mock".to_string(),
            release_version: "1.0".to_string(),
            source_path: PathBuf::from("/tmp/mock.iso"),
            target_device: None,
        }
    }

    #[test]
    fn registry_finds_registered_backend_by_id() {
        let mut registry = BackendRegistry::new();
        registry.register(Box::new(MockBackend));

        let backend = registry.get("mock").unwrap();
        assert_eq!(backend.id(), "mock");
        assert!(backend.validate(&ctx()).unwrap().ok);
    }

    #[test]
    fn registry_reports_unknown_backend() {
        let registry = BackendRegistry::new();
        assert!(matches!(
            registry.get("does-not-exist"),
            Err(InstallerError::UnknownBackend(_))
        ));
    }
}
