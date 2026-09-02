# Security model

Universal Net Installer will eventually erase disks. That is the single
most dangerous thing this project does, so the rules here are treated as
load-bearing, not aspirational.

## The forbidden list (phase 1)

No code in this repository invokes any of:

```
dd  wipefs  mkfs  fdisk  parted  sgdisk
```

This isn't just a coding-style rule enforced by review: as of phase 1
there is **no function anywhere in the workspace that could invoke them**
— `uni_storage::StorageGuard` (the only type designed to sit in front of a
destructive operation) has `validate()` and `confirmation_prompt()` and
deliberately no `execute()`. Adding real destructive execution is future
work that gets its own review, its own tests, and its own entry in
`docs/roadmap.md` — it does not sneak in as a side effect of another
feature.

## `StorageGuard`

Every destructive operation, whenever it is implemented, must go through
`uni_storage::StorageGuard`:

1. `validate(disks, request)` — confirms the target device exists in the
   current inventory and is not `Protected`. Purely a check: no I/O
   against the device.
2. `confirmation_prompt(disk, request)` — renders the mandatory warning
   block the UI must show, verbatim, before a user can confirm:

   ```
   WARNING

   Device:
   /dev/nvme0n1

   Model:
   Samsung 990 Pro

   Serial:
   S6XPNS0R999999

   Size:
   2.0 TB

   Partitions:
   /dev/nvme0n1p1, /dev/nvme0n1p2

   Action:
   ERASE ENTIRE DISK
   ```

A UI is never allowed to skip straight to an action string typed by the
user; it renders this struct's output and requires an explicit,
unambiguous confirmation (not a bare Enter-to-accept default).

## `PROTECTED`: never destroy your own boot medium

`uni_storage::boot_device` identifies the disk Universal Net Installer
itself booted from (via `findmnt` on the live medium mountpoint, falling
back to `/`) and marks it `ProtectionState::Protected` in the disk
inventory `uni_hardware::detect()` returns. `StorageGuard::validate`
refuses any request whose target is `Protected`, unconditionally — there
is no override flag. A user cannot accidentally (or deliberately, through
the TUI) select the USB stick they're running from as an install/erase
target. See [`storage-safety.md`](storage-safety.md) for the detection
mechanism in detail.

## Secrets never reach the logs

Wi-Fi passwords are the one credential this project handles. They travel
from the TUI's input prompt straight into
`NetworkManagerBackend::connect_wifi`, which — whenever a password is
present — calls `uni_core::process::run_redacted` instead of `run`.
`run_redacted` logs the command name (`nmcli`) at `debug` level and
nothing else; the full argument list, which is where the password would
appear, is never formatted into a log line. There is no other code path
in the workspace that takes a password as input.

## Integrity before execution

`uni-verifier` implements SHA-256 today; SHA-512 and GPG signature
verification are modeled in `VerificationMethod` so callers and manifests
can already reference them, but dispatching to either returns
`VerifierError::NotImplemented` — there is no silent "skip verification"
path. `uni_downloader::Downloader::download_and_verify_sha256` is the
intended entry point for fetching a distribution artifact specifically
*because* it composes the download with verification in one call: the
only `Ok` result is "downloaded and verified." The lower-level
`Downloader::download` (no verification) exists for cases that
genuinely don't need it (e.g. fetching a checksum file itself), and
callers are expected to route anything that will be handed to an
installer through the verified path.

## What phase 1 deliberately does not do

- Does not download any distribution ISO or resource.
- Does not launch any installer.
- Does not perform GPG verification (designed, not implemented).
- Does not implement SHA-512 (designed, not implemented).
- Does not implement any concrete `InstallerBackend` (Ubuntu, Debian,
  Fedora, Arch) — the trait and registry exist, nothing implements it yet.

## Threat model notes for later phases

- **Manifests are data, not code.** `uni-catalog` parses YAML into typed
  structs via `serde`; nothing in the manifest is ever passed to a shell.
  A malicious or corrupted manifest can at worst point the downloader at
  the wrong URL — which verification is designed to catch before that
  artifact is trusted.
- **Mirror URLs must be HTTPS.** The downloader's mirror list is a
  `Vec<String>` of URLs; certificate validation is `reqwest`'s default
  (via `rustls`), not disabled anywhere in this codebase.
- **The destructive-execution phase**, when it lands, will need: an
  explicit user confirmation UI that echoes `StorageGuard`'s prompt
  verbatim, a re-check of `validate()` immediately before the destructive
  call (not just when the screen was first shown), and its own security
  review pass. That work is out of scope for this repository until a
  dedicated roadmap phase opens it.
