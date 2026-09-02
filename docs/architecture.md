# Architecture

## Goals that shaped it

- **Business logic never depends on the UI.** `uni-tui` is the only crate
  that imports `ratatui`/`crossterm`. Every other crate is a plain library
  usable from a future GUI (Slint), a test harness, or a headless CLI
  without change.
- **No fragile text scraping when a stable machine-readable form exists.**
  `lsblk -J`, `lspci -mm`, `nmcli -t` and `/sys`/`/proc` are used instead
  of parsing human-formatted tables. See each crate's module docs for the
  specific format chosen and why.
- **Destructive capability is opt-in and reviewable, not ambient.** Any
  crate that could theoretically touch a disk goes through
  `uni_storage::StorageGuard`; see [`security.md`](security.md).
- **The platform is x86_64/UEFI today, without hard-coding that fact.**
  `uni_core::Architecture` and `uni_hardware::BootMode` are already enums,
  not string literals, so adding `Aarch64` or `BootMode::Bios` handling
  later touches match arms, not APIs.

## Crate dependency graph

```
uni-core
  ▲   ▲    ▲
  │   │    │
uni-storage   uni-network
  ▲       ▲     ▲
  │       └──┬──┘
  │      uni-hardware
  │
uni-verifier
  ▲
  │
uni-downloader          uni-catalog
                              ▲
                              │
uni-installer ───────────────┘ (uni-storage only, for StorageGuard)

uni-tui → uni-core, uni-hardware, uni-storage, uni-network
```

(`uni-installer` does not depend on `uni-catalog` in phase 1: its
`InstallContext` takes plain strings/paths so the two crates can evolve
independently until a concrete backend needs manifest data.)

`uni-core` has no workspace dependencies — it's the only crate every other
crate can assume is always available.

## Why `uni-hardware` depends on `uni-storage` and `uni-network`

The spec for hardware detection lists "disks" and "network interfaces" as
things it reports, and `uni-storage`/`uni-network` separately own deep,
domain-specific APIs (`StorageGuard`, `NetworkBackend`) for those same
devices. Rather than parse `lsblk`/`/sys/class/net` a second time inside
`uni-hardware`, it composes the other crates' detection functions:
`uni_hardware::HardwareSnapshot` is a facade that aggregates CPU/RAM/GPU/
boot-mode (which it does own) with `Vec<uni_storage::DiskInfo>` and
`Vec<uni_network::Interface>` (which it borrows). One parser per data
source, one snapshot type for the TUI to render.

## Error handling

Every crate defines its own `thiserror` enum (`StorageError`,
`NetworkError`, `HardwareError`, ...) rather than sharing one giant error
type. `uni_core::CoreError` covers the primitives every crate is built on
(I/O, process execution) and is wrapped via `#[from]` into each domain
error, so `?` composes across crate boundaries without manual mapping at
most call sites. `anyhow` is reserved for `uni-tui`'s `main`, where the
consumer is a human reading a message, not code branching on a variant.

## Process execution

`uni_core::process::run` and `run_redacted` are the only sanctioned way to
shell out in this workspace. Every command's arguments are logged at
`debug` *except* through `run_redacted`, which logs only the command name
— used by `uni-network` for `nmcli device wifi connect ... password ...`
so a Wi-Fi password can never reach the logs. See
[`network.md`](network.md) and [`security.md`](security.md).

## Data flow in phase 1

```
uni-tui main()
  → App::refresh()
    → uni_hardware::detect()
        → uni_hardware::cpu::detect_cpu()        (/proc/cpuinfo)
        → uni_hardware::memory::detect_memory()  (/proc/meminfo)
        → uni_hardware::gpu::detect_gpus()        (lspci -mm, best-effort)
        → uni_hardware::boot_mode::detect_boot_mode() (/sys/firmware/efi)
        → uni_storage::detect_disks()             (lsblk -J -b -O, findmnt)
        → uni_network::detect_interfaces()        (/sys/class/net)
  → ui::draw(frame, app)   (ratatui render, no side effects)
```

Nothing in this call graph performs a network request, writes to a disk,
or launches a subprocess other than the read-only detection commands
above.

## Extension points already in place

- **BIOS/legacy support**: `uni_hardware::BootMode` already has a `Bios`
  variant; `docs/boot-process.md` documents what changes when GRUB2 boots
  via legacy BIOS instead of UEFI.
- **ARM64**: `uni_core::Architecture::Aarch64` exists; `uni-catalog`
  manifests already carry a free-form `architecture` string per release
  rather than assuming x86_64.
- **GUI**: because `uni-tui` only calls public APIs on the domain crates
  (never reaches into their internals), a `uni-gui` crate built on Slint
  can be added as a sibling that depends on the same `uni-hardware`,
  `uni-network`, etc. without any of them changing.
- **Metalink/torrent downloads**: `uni_downloader::DownloadRequest` takes
  a `Vec<String>` of HTTPS mirrors today; a new `SourceKind` in
  `uni-catalog` plus a new request variant is additive, not a rewrite.

## Directory layout

```
crates/            Rust workspace members (this document's subject)
manifests/          Distribution catalog (docs/manifests.md)
live/               Debian Live (live-build) tree — populated in Fase 10
scripts/            build-live.sh / build-usb.sh / test-qemu.sh — Fase 10+
docs/               This directory
.github/workflows/  CI (fmt, clippy, test, build — no ISO publishing yet)
```
