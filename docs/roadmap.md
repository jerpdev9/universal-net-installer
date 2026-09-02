# Roadmap

Status legend: ✅ done · 🚧 partial · ⬜ not started

| Phase | Scope | Status |
|---|---|---|
| 0 | Architecture and documentation | ✅ this document, `PLAN.md`, `architecture.md`, `security.md` |
| 1 | Cargo Workspace | ✅ 9 crates, workspace-shared dependency versions |
| 2 | TUI | ✅ dashboard shell (`uni-tui`) plus Wi-Fi and OS-catalog popup flows |
| 3 | Hardware detection | ✅ CPU, RAM, GPU, boot mode, disks (`uni-hardware`, `uni-storage`) |
| 4 | Networking | ✅ interface detection, `NetworkBackend` trait + `nmcli` backend (`uni-network`) |
| 5 | Wi-Fi | ✅ `scan_wifi`/`connect_wifi` wired into `uni-tui`: `w` scans, arrow keys + Enter pick a network, a password popup handles secured networks, a connectivity indicator shows `Internet: Online/Limited/Offline/Unknown` |
| 6 | Catalog YAML | ✅ schema + loader (`uni-catalog`), 4 manifests, wired into `uni-tui`: `o` browses distributions, Enter drills into releases; picking a release only shows a status message, nothing downloads |
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

**Phase 7 (Downloader) → TUI integration.** The catalog now gives the TUI
a concrete `Release` to act on (`Source::resolve_url`), and
`uni-downloader`'s `Downloader::download_and_verify_sha256` already
composes fetch + integrity check into one call. The smallest useful next
step is a progress screen: pick a release (as today), resolve its mirror
URL, download to a scratch path with a live progress bar
(`ProgressSink`), and verify it — surfacing success/failure in the status
line the same way Wi-Fi and catalog errors already do. This is the first
phase that touches the network for something other than connectivity
checks and Wi-Fi scanning, so it's worth its own review pass before
merging: in particular, where the download's scratch path lives and that
nothing downstream treats an unverified file as usable. Still nothing
that opens the destructive-storage or install work.
