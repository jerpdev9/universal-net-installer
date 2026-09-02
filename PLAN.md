# Universal Net Installer — Plan

## What this project is

A bootable USB environment that starts on x86_64/UEFI hardware, brings up
networking (Ethernet or Wi-Fi), lets the user browse a YAML-defined catalog
of Linux distributions, downloads and verifies the chosen release, and
launches that distribution's **own official installer**. Universal Net
Installer is a launcher and a safety layer in front of installers, not a
reimplementation of them.

Target platform for the MVP: **x86_64, UEFI**. The crate boundaries are
drawn so BIOS (legacy) and ARM64 can be added later without reshaping the
workspace — see [`docs/architecture.md`](docs/architecture.md).

## MVP flow

1. Boot the USB (Debian Live + GRUB2, UEFI).
2. `uni-tui` starts automatically.
3. Hardware is detected: CPU, RAM, GPU, disks, NVMe, Ethernet, Wi-Fi,
   UEFI/BIOS mode.
4. The user picks a Wi-Fi network and enters a password (or plugs in
   Ethernet).
5. Connectivity is confirmed.
6. The catalog (`manifests/*.yaml`) is loaded and shown.
7. The user picks a distribution and a release.
8. The ISO/kernel+initrd is downloaded (resumable, mirror-aware).
9. SHA-256 (and GPG, where published) is verified.
10. The distribution's official installer is launched.

Windows is out of scope for the MVP.

## Phase 1 — what this delivers

This repository currently implements **only** the read-only half of that
flow: hardware detection and its TUI presentation. Concretely:

- The full Cargo workspace (9 crates) with real dependency boundaries.
- `uni-core`, `uni-storage`, `uni-network`, `uni-hardware`: fully
  implemented and tested.
- `uni-tui`: a working dashboard (`cargo run -p uni-tui`) showing CPU, RAM,
  GPU, boot mode, disks (with the boot USB marked `PROTECTED`) and network
  interfaces.
- `uni-catalog`, `uni-verifier`, `uni-downloader`, `uni-installer`: the
  designed APIs (manifest schema + loader, SHA-256 verification, a
  resumable/mirror-aware downloader, the `InstallerBackend` trait +
  registry) exist and are unit-tested, but nothing in the workspace calls
  them yet. No network download and no installer launch happens as a side
  effect of running `uni-tui`.
- Four real manifests (`manifests/{ubuntu,debian,fedora,arch}.yaml`).

**Explicitly not present in this phase**, by design (see
[`docs/security.md`](docs/security.md)):

- No code path calls `dd`, `wipefs`, `mkfs`, `fdisk`, `parted` or
  `sgdisk`. `uni_storage::StorageGuard` only validates and renders a
  confirmation prompt — it has no `execute()` method.
- No ISO is downloaded by anything the TUI does.
- No installer is launched.

## Phase 2 — Wi-Fi connect flow

Added on top of phase 1, following the roadmap's suggested next step:
`uni-tui` now has a Wi-Fi scan/connect flow built on `uni-network`'s
already-implemented `NetworkManagerBackend`. Pressing `w` scans the
detected Wi-Fi interface; arrow keys (or `j`/`k`) and Enter pick a
network; open networks connect immediately, secured ones prompt for a
password in a masked popup; `Esc` cancels back to the dashboard at any
point. The network panel also shows a live `Internet:
Online/Limited/Offline/Unknown` connectivity indicator. Still true: no
download, no disk modification, no installer launch — this phase only
exercises `NetworkBackend`, which `docs/network.md` already documented as
safe (NetworkManager owns the WPA handshake; passwords never reach the
logs).

## Phase 3 — catalog browsing

Added on top of phase 2, following the roadmap's suggested next step:
`uni-tui` now loads `manifests/*.yaml` at startup via
`uni_catalog::load_catalog_dir` and lets the user browse it. Pressing `o`
opens the distribution list (name, vendor, release count); Enter drills
into that distribution's releases (version, architecture, source kind,
installer backend); Enter on a release records the choice in the status
line — `"selected Ubuntu latest-lts — download not implemented yet
(docs/roadmap.md phase 7)"` — and returns to the dashboard. `Esc` cancels
back a level at any point, same as the Wi-Fi flow. Still true: no
download, no disk modification, no installer launch — this phase only
reads and displays already-shipped YAML files.

## Crate map

| Crate | Responsibility | Status |
|---|---|---|
| `uni-core` | Errors, instrumented process execution, logging, `Architecture` | Implemented |
| `uni-storage` | Disk/partition detection, boot-device id, `StorageGuard` (no execution) | Implemented |
| `uni-network` | Interface detection, `NetworkBackend` (nmcli-backed) | Implemented |
| `uni-hardware` | CPU/RAM/GPU/boot-mode detection, composes storage+network into `HardwareSnapshot` | Implemented |
| `uni-tui` | Ratatui dashboard + Wi-Fi scan/connect + catalog browse flows | Implemented (detection + Wi-Fi + catalog) |
| `uni-catalog` | YAML manifest schema + loader | Implemented, wired into `uni-tui` |
| `uni-verifier` | SHA-256 (SHA-512/GPG designed, not implemented) | Implemented, not wired in |
| `uni-downloader` | Resumable, mirror-aware HTTPS download + progress/cancel | Implemented, not wired in |
| `uni-installer` | `InstallerBackend` trait + registry, no concrete backends | Implemented, not wired in |

See [`docs/architecture.md`](docs/architecture.md) for the dependency graph
and design rationale, [`docs/security.md`](docs/security.md) for the
safety model, and [`docs/roadmap.md`](docs/roadmap.md) for what comes
next.

## How to verify this phase

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo build --workspace
cargo run -p uni-tui
# q quit, r refresh, w scan Wi-Fi (Enter connect, Esc cancel)
# o browse OS catalog (Enter drills in, Esc cancels)
```
