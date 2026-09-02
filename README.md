# Universal Net Installer

A bootable USB environment that boots a PC, connects to the Internet over
Ethernet or Wi-Fi, and lets you select, download, verify and launch the
official installer of a Linux distribution.

## Goals

- Boot from USB (x86_64, UEFI initially; BIOS and ARM64 planned)
- Detect hardware: CPU, RAM, GPU, disks/NVMe, network interfaces, boot mode
- Connect through Ethernet or Wi-Fi (via NetworkManager)
- Read a YAML catalog of Linux distributions — nothing hardcoded in Rust
- Download and verify (SHA-256, GPG) installer resources before use
- Launch each distribution's own official installer
- Protect storage devices against accidental destructive operations

Windows is out of scope for the MVP.

## Status: hardware detection + Wi-Fi connect + catalog browsing

The Cargo workspace, its 9 crates and their unit tests are implemented.
`uni-tui` runs and shows real CPU/RAM/GPU/boot-mode/disk/network data, can
scan for and connect to Wi-Fi networks through NetworkManager, and lets
you browse the distribution catalog (`manifests/*.yaml`). Nothing
downloads an ISO, modifies a disk, or launches an installer yet — see
[`PLAN.md`](PLAN.md) for exactly what's covered and
[`docs/roadmap.md`](docs/roadmap.md) for what's next.

```bash
cargo run -p uni-tui
# q quit, r refresh, w scan Wi-Fi (Enter connect, Esc cancel)
# o browse OS catalog (Enter drills in, Esc cancels)
```

## Documentation

- [`PLAN.md`](PLAN.md) — what this phase delivers, crate-by-crate
- [`docs/architecture.md`](docs/architecture.md) — crate graph and design rationale
- [`docs/security.md`](docs/security.md) — the destructive-operation safety model
- [`docs/storage-safety.md`](docs/storage-safety.md) — disk detection, boot-device protection, `StorageGuard`
- [`docs/network.md`](docs/network.md) — interface detection and the `NetworkBackend` abstraction
- [`docs/manifests.md`](docs/manifests.md) — the distribution catalog schema
- [`docs/boot-process.md`](docs/boot-process.md) — how the live USB boots and how it will hand off to installers
- [`docs/roadmap.md`](docs/roadmap.md) — phase-by-phase status

## Development

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo build --workspace
```

## Safety

This project will eventually perform destructive disk operations. As of
this phase, no code path can: there is no `dd`, `wipefs`, `mkfs`, `fdisk`,
`parted` or `sgdisk` invocation anywhere in the workspace, and
`StorageGuard` (the type any future destructive operation must go through)
has no `execute()` method. See [`docs/security.md`](docs/security.md).

## License

To be defined.
