# Storage safety

This is the detail behind the rules stated in [`security.md`](security.md):
how disk detection, boot-device identification and `StorageGuard` actually
work.

## Detection: `lsblk -J -b -O`

`uni_storage::detect_disks` shells out to `lsblk -J -b -O` (JSON output,
byte-precise sizes, every optional column) instead of parsing the
human-readable table. The JSON schema is stable across `util-linux`
versions and distributions in a way column widths are not — see
`uni-storage/src/disk.rs` for the parser, which is pure and unit-tested
against fixture JSON.

Each disk is classified into a `DiskKind` from its `tran` (transport) and
`rota` (rotational) fields:

| `tran` | `rota` | `DiskKind` |
|---|---|---|
| `nvme` | any | `Nvme` |
| `usb` | any | `Usb` |
| `sata`/`ata`/`sas`/`scsi` | `1` | `Hdd` |
| `sata`/`ata`/`sas`/`scsi` | `0` | `SataSsd` |
| anything else | — | `Unknown` |

A device also gets forced to `Usb` if the kernel reports it removable
(`rm=1`), regardless of transport, except NVMe (there is no such thing as
a removable NVMe stick in practice, and some external NVMe enclosures
still report `tran=nvme`).

## Boot-device identification

`uni_storage::boot_device::mark_boot_device` resolves the disk Universal
Net Installer itself booted from:

1. Try `findmnt -no SOURCE /run/live/medium` — the mountpoint
   `live-boot` (Debian Live) creates for the medium it booted from.
2. Fall back to `findmnt -no SOURCE /` — a plain installed system, or a
   live environment whose layout differs. Useful mainly during
   development, since a live root is usually an overlay, not a block
   device.
3. If the resolved source isn't a `/dev/...` path (e.g. `overlay`,
   `tmpfs`), give up rather than guess. No disk is marked `Protected` in
   that case — a false negative (nothing protected) is the safe failure
   mode; a false positive (wrong disk protected) would not be.
4. Strip the partition suffix to get the parent disk name:
   `/dev/sdb1 → sdb`, `/dev/nvme0n1p1 → nvme0n1`,
   `/dev/mmcblk0p1 → mmcblk0`. This mapping (`partition_to_disk_name`) is
   pure and unit-tested.

The matching `DiskInfo` in the inventory gets
`protection = ProtectionState::Protected`.

## `StorageGuard`

See [`security.md`](security.md#storageguard) for the full description.
In short: `validate()` checks existence and protection status;
`confirmation_prompt()` renders the mandatory warning block; there is no
`execute()` in this phase, by design — see
[`security.md`](security.md#the-forbidden-list-phase-1).

## What this buys the rest of the workspace

Because `uni-hardware` composes `uni_storage::detect_disks()` into
`HardwareSnapshot`, every disk the TUI ever displays — now, and in every
future phase that adds an install-target picker — already carries its
`ProtectionState`. A future selection screen only needs to grey out or
hide `Protected` entries; it never needs to re-derive "is this the boot
USB" itself.
