//! Schema for `manifests/*.yaml`. No distribution or version number is
//! hardcoded in Rust anywhere in this crate — every fact about "which
//! releases exist" lives in the YAML. See `docs/manifests.md`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub id: String,
    pub name: String,
    pub vendor: String,
    pub homepage: String,
    pub releases: Vec<Release>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Release {
    /// Free-form release identifier, e.g. `"latest-lts"`, `"stable"`,
    /// `"rolling"`, or a concrete version. Never matched on in Rust code.
    pub version: String,
    pub architecture: String,
    pub source: Source,
    pub verification: Verification,
    pub installer: InstallerRef,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceKind {
    Iso,
    Netboot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    #[serde(rename = "type")]
    pub kind: SourceKind,
    /// Mirror base URLs, in priority order.
    pub mirrors: Vec<String>,
    /// Path appended to a mirror to form the download URL. May contain a
    /// `{version}` placeholder, substituted by [`Source::resolve_url`].
    pub path: String,
    /// Set for `netboot` releases that boot via kernel + initrd instead of
    /// a full ISO.
    #[serde(default)]
    pub kernel: Option<String>,
    #[serde(default)]
    pub initrd: Option<String>,
}

impl Source {
    /// Builds the download URL for `mirror_index`, substituting
    /// `{version}` in `path`. Returns `None` if `mirror_index` is out of
    /// range.
    pub fn resolve_url(&self, mirror_index: usize, version: &str) -> Option<String> {
        let mirror = self.mirrors.get(mirror_index)?;
        let path = self.path.replace("{version}", version);
        Some(format!(
            "{}/{}",
            mirror.trim_end_matches('/'),
            path.trim_start_matches('/')
        ))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Verification {
    #[serde(default)]
    pub sha256_path: Option<String>,
    #[serde(default)]
    pub gpg_signature_path: Option<String>,
    #[serde(default)]
    pub gpg_key_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallerRef {
    /// Matches an `InstallerBackend::id()` in `uni-installer`, e.g.
    /// `"ubuntu"`, `"debian"`, `"fedora"`, `"arch"`.
    pub backend: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_url_substitutes_version_and_joins_mirror() {
        let source = Source {
            kind: SourceKind::Iso,
            mirrors: vec!["https://example.com/releases/".to_string()],
            path: "{version}/distro-{version}.iso".to_string(),
            kernel: None,
            initrd: None,
        };
        assert_eq!(
            source.resolve_url(0, "24.04").unwrap(),
            "https://example.com/releases/24.04/distro-24.04.iso"
        );
    }

    #[test]
    fn resolve_url_returns_none_for_out_of_range_mirror() {
        let source = Source {
            kind: SourceKind::Iso,
            mirrors: vec![],
            path: "x".to_string(),
            kernel: None,
            initrd: None,
        };
        assert!(source.resolve_url(0, "1").is_none());
    }
}
