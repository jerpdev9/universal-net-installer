# Roadmap

Status legend: ✅ done · 🚧 partial · ⬜ not started

| Phase | Scope | Status |
|---|---|---|
| 0 | Architecture and documentation | ✅ this document, `PLAN.md`, `architecture.md`, `security.md` |
| 1 | Cargo Workspace | ✅ 9 crates, workspace-shared dependency versions |
| 2 | TUI | ✅ dashboard shell (`uni-tui`); no menu/selection screens yet |
| 3 | Hardware detection | ✅ CPU, RAM, GPU, boot mode, disks (`uni-hardware`, `uni-storage`) |
| 4 | Networking | ✅ interface detection, `NetworkBackend` trait + `nmcli` backend (`uni-network`) |
| 5 | Wi-Fi | 🚧 `scan_wifi`/`connect_wifi` implemented in `uni-network`; no TUI screen calls them yet |
| 6 | Catalog YAML | ✅ schema + loader (`uni-catalog`), 4 manifests; not loaded by the TUI yet |
| 7 | Downloader | 🚧 resumable/mirror-aware API implemented (`uni-downloader`); nothing invokes it |
| 8 | SHA256 | ✅ implemented and tested (`uni-verifier`) |
| 9 | GPG | ⬜ modeled in `VerificationMethod::Gpg`, returns `NotImplemented` |
| 10 | Debian Live | ⬜ `live/` directory scaffold only |
| 11 | Boot UEFI | ⬜ `docs/boot-process.md` describes the target mechanism; no GRUB2 config yet |
| 12 | Ubuntu Installer | ⬜ `InstallerBackend` trait exists (`uni-installer`); no concrete backend |
| 13 | Debian Installer | ⬜ same |
| 14 | Fedora | ⬜ same |
| 15 | Arch | ⬜ same |
| 16 | GUI | ⬜ architecture supports adding a `uni-gui` (Slint) crate; not started |
| 17 | Secure Boot | ⬜ not started |

## What "done" means for phase 1 specifically

The acceptance criteria for this phase were: create the architecture docs,
create the Cargo workspace, and implement a safe prototype where
`cargo run -p uni-tui` shows CPU, RAM, disks, network interfaces and
UEFI/BIOS mode — without downloading any ISO, modifying any disk, or
launching any installer. All of that is met; see `PLAN.md` for the
per-crate breakdown and `docs/security.md` for what remains deliberately
absent.

## Suggested next phase

**Phase 5 (Wi-Fi) → TUI integration**, since the backend already exists:
add a network-selection screen to `uni-tui` that calls
`NetworkManagerBackend::scan_wifi`/`connect_wifi`, and a connectivity
indicator using `connectivity()`. This is the smallest phase that turns
already-implemented, already-tested library code into something a user
can actually drive, without opening any of the destructive-storage or
download/verify/install work yet.
