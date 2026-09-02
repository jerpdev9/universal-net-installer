# Boot process

This covers two different boots that must not be confused: how the
Universal Net Installer **live USB itself** boots, and how it hands off
to a **target distribution's own installer**. The second is design
guidance for roadmap phases 11-15 (`docs/roadmap.md`) — nothing described
under "Launch mechanisms per distribution" is implemented yet.

## How Universal Net Installer boots

Target for the MVP: **x86_64, UEFI**.

1. Firmware (UEFI) loads GRUB2's EFI binary from the USB's EFI System
   Partition (`/EFI/BOOT/BOOTX64.EFI` or a signed shim, if Secure Boot
   support lands per `docs/roadmap.md` phase 17).
2. GRUB2 loads the Debian Live kernel and initrd from the USB.
3. The kernel boots into a `live-boot`/`live-config` environment (Debian
   Live via `live-build`, `live/` in this repo).
4. `live-config` runs `uni-tui` as the environment's entry point (see
   `live/hooks/`, populated in phase 10).
5. `uni_hardware::detect_boot_mode` confirms UEFI by checking
   `/sys/firmware/efi` exists — this is also how the rest of the codebase
   knows which mode it's running in; see `uni-hardware/src/boot_mode.rs`.

Legacy BIOS support means step 1 uses GRUB2's BIOS boot path (MBR + boot
flag) instead of an EFI binary; `uni_hardware::BootMode::Bios` already
exists so the rest of the stack doesn't need to change when that's added.

## Launch mechanisms — the vocabulary

Four different ways Universal Net Installer could hand off to a target
distribution's installer, in increasing order of "how much we get to
avoid re-implementing":

- **Live installer (ISO loopback)**: mount/loopback the downloaded ISO
  and either `kexec` into its own kernel+initrd, or add a GRUB2
  `loopback.cfg` boot entry pointing at the ISO file so the *next* boot
  starts the official live environment and its installer directly. No
  network fetch of a separate kernel — the ISO is the whole payload.
- **kexec**: load the target's kernel+initrd (extracted from the ISO, or
  fetched directly for distros that publish them standalone) into memory
  and jump to it without a firmware reboot. Faster than a full reboot,
  and avoids depending on the USB still being bootable/present after
  Universal Net Installer's own kernel exits — relevant since a user's
  target install disk might be the same physical bus.
- **netboot (kernel + initrd only)**: some distributions publish a small
  installer kernel+initrd that itself pulls packages over the network
  during install, instead of shipping a full ISO. `uni-catalog`'s
  `SourceKind::Netboot` and the `kernel`/`initrd` manifest fields exist
  for this case.
- **Chainloading**: GRUB2 boots another bootloader/binary (e.g. the
  target ISO's own `isolinux`/`grub` config) rather than a kernel
  directly. Used when a distribution's boot chain expects to run its own
  bootloader logic (e.g. its own memtest/boot-parameter menu) rather than
  being handed a raw kernel+initrd.

## Launch mechanisms per distribution (planned)

| Distribution | Expected mechanism | Why |
|---|---|---|
| Ubuntu | Live installer (ISO loopback) or kexec into the ISO's `casper` kernel+initrd | Ubuntu ships a single desktop ISO; no standalone netboot kernel+initrd is published for the desktop installer |
| Debian | netboot (kernel + initrd) where available, else live installer (ISO loopback) | Debian publishes dedicated `netboot` mini-images with `debian-installer`, a better fit than downloading the full ISO first |
| Fedora | netboot (kernel + initrd) via the Anaconda `boot.iso`/netinstall image | Fedora's Anaconda installer is designed to boot from a small kernel+initrd and fetch packages afterward |
| Arch | Live installer (ISO loopback) or kexec into `archiso`'s kernel+initrd | Arch ships one ISO with no separate netboot artifact; installation itself is scripted (`archinstall`) after boot, not a distinct "installer" binary |

Each concrete `InstallerBackend` (phases 12-15) will confirm the mechanism
against the actual current release layout at implementation time — this
table is a starting hypothesis from each project's publicly documented
boot artifacts, not a verified spec. `InstallContext::source_path` and the
manifest's `Source::kernel`/`Source::initrd` fields are already shaped to
carry either an ISO path or a kernel+initrd pair, so a backend can pick
whichever mechanism its distribution actually needs without a schema
change.

## What never happens regardless of mechanism

No launch mechanism above writes to a disk. `kexec` and chainloading both
replace the *currently running* kernel/bootloader in memory; they are
independent from, and must never be confused with, the disk-partitioning
and filesystem work `uni_storage::StorageGuard` gates (`docs/security.md`).
The target distribution's own installer — not Universal Net Installer —
is what eventually asks the user to pick a disk and performs that
destructive work, using its own UI.
