//! Loads [`Manifest`]s from `manifests/*.yaml`.

use std::fs;
use std::path::Path;

use crate::error::{CatalogError, Result};
use crate::manifest::Manifest;

/// Parses a single manifest from a YAML string.
pub fn load_from_str(yaml: &str) -> Result<Manifest> {
    let manifest: Manifest = serde_yaml::from_str(yaml).map_err(|e| CatalogError::Parse {
        path: "<string>".to_string(),
        reason: e.to_string(),
    })?;
    validate(&manifest, "<string>")?;
    Ok(manifest)
}

/// Loads and validates a single manifest file.
pub fn load_from_path(path: &Path) -> Result<Manifest> {
    let raw = fs::read_to_string(path).map_err(|source| {
        CatalogError::Core(uni_core::CoreError::Io {
            path: path.to_path_buf(),
            source,
        })
    })?;
    let manifest: Manifest = serde_yaml::from_str(&raw).map_err(|e| CatalogError::Parse {
        path: path.display().to_string(),
        reason: e.to_string(),
    })?;
    validate(&manifest, &path.display().to_string())?;
    Ok(manifest)
}

/// Loads every `*.yaml` manifest in `dir` (non-recursive), sorted by id.
pub fn load_catalog_dir(dir: &Path) -> Result<Vec<Manifest>> {
    let mut entries: Vec<_> = fs::read_dir(dir)
        .map_err(|source| {
            CatalogError::Core(uni_core::CoreError::Io {
                path: dir.to_path_buf(),
                source,
            })
        })?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("yaml"))
        .collect();
    entries.sort_by_key(|e| e.path());

    let mut manifests = Vec::with_capacity(entries.len());
    for entry in entries {
        manifests.push(load_from_path(&entry.path())?);
    }
    manifests.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(manifests)
}

fn validate(manifest: &Manifest, path: &str) -> Result<()> {
    if manifest.releases.is_empty() {
        return Err(CatalogError::EmptyReleases {
            path: path.to_string(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifests_dir() -> std::path::PathBuf {
        // crates/uni-catalog -> repo root -> manifests/
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../manifests")
    }

    #[test]
    fn loads_all_shipped_manifests() {
        let manifests = load_catalog_dir(&manifests_dir()).unwrap();
        let ids: Vec<_> = manifests.iter().map(|m| m.id.as_str()).collect();
        assert_eq!(ids, vec!["arch", "debian", "fedora", "ubuntu"]);
        for manifest in &manifests {
            assert!(
                !manifest.releases.is_empty(),
                "{} has releases",
                manifest.id
            );
        }
    }

    #[test]
    fn rejects_manifest_with_no_releases() {
        let yaml =
            "id: empty\nname: Empty\nvendor: Nobody\nhomepage: https://example.com\nreleases: []\n";
        let err = load_from_str(yaml).unwrap_err();
        assert!(matches!(err, CatalogError::EmptyReleases { .. }));
    }

    #[test]
    fn rejects_invalid_yaml() {
        let err = load_from_str("not: [valid, yaml: structure").unwrap_err();
        assert!(matches!(err, CatalogError::Parse { .. }));
    }
}
