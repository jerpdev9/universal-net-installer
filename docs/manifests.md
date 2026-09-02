# Distribution manifests

No distribution or version number is hardcoded in Rust anywhere in this
workspace. Everything `uni-catalog` knows about Ubuntu, Debian, Fedora and
Arch comes from `manifests/*.yaml`, loaded at runtime by
`uni_catalog::load_catalog_dir`.

## Schema

```yaml
id: ubuntu                  # stable identifier, matched against installer.backend elsewhere
name: Ubuntu
vendor: Canonical
homepage: "https://ubuntu.com"

releases:
  - version: "latest-lts"    # free-form: "latest-lts", "stable", "rolling", or a concrete version
    architecture: x86_64
    source:
      type: iso              # iso | netboot
      mirrors:
        - "https://releases.ubuntu.com"
      path: "{version}/ubuntu-{version}-desktop-amd64.iso"   # {version} is substituted
      kernel: null            # set for netboot releases
      initrd: null
    verification:
      sha256_path: "{version}/SHA256SUMS"
      gpg_signature_path: "{version}/SHA256SUMS.gpg"
      gpg_key_url: "https://ubuntu.com/gpg"
    installer:
      backend: ubuntu         # matches an InstallerBackend::id() in uni-installer
```

`Source::resolve_url(mirror_index, version)` substitutes `{version}` into
`path` and joins it to the chosen mirror, trying mirrors in the order
listed. `verification.*_path` fields follow the same `{version}`
convention conceptually, though the substitution helper today only lives
on `Source` — extending it to `Verification` is straightforward when the
downloader phase actually needs it.

## Why `version` is a string, not a number

`"latest-lts"`, `"stable"` and `"rolling"` are what the shipped manifests
actually use — Debian's stable name and Arch's rolling releases don't have
a single "version number" to pin in the first place, and Ubuntu/Fedora's
concrete version numbers go stale the moment a new release ships. Keeping
`version` free-form means updating a manifest file is how a new release
becomes available — no Rust code change, no rebuild.

## The four shipped manifests

| id | version field | source type | notes |
|---|---|---|---|
| `ubuntu` | `latest-lts` | iso | `releases.ubuntu.com` |
| `debian` | `stable` | iso | `cdimage.debian.org` |
| `fedora` | `latest` | iso | `download.fedoraproject.org`; no published GPG signature file, only a `CHECKSUM` |
| `arch` | `rolling` | iso | `geo.mirror.pkgbuild.com`; Arch ships one always-current ISO name |

These mirror URLs and path templates are illustrative of each project's
real download layout, not resolved or fetched by anything in phase 1 —
see `docs/roadmap.md` phase 7.

## Loading and validation

`uni_catalog::load_from_path`/`load_from_str` parse via `serde_yaml` and
reject a manifest with an empty `releases` list
(`CatalogError::EmptyReleases`). `load_catalog_dir` loads every `*.yaml`
in a directory and returns manifests sorted by `id`. All four shipped
manifests round-trip through this loader in
`uni-catalog/src/loader.rs`'s test suite.

## Adding a new distribution

Add a new `manifests/<id>.yaml` following the schema above. No Rust
change is required until a concrete `InstallerBackend` for it exists
(`docs/roadmap.md` phases 12-15) — the manifest, the catalog loader and
the downloader are all distribution-agnostic already.
