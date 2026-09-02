# Universal Net Installer

Bootable USB environment for installing Linux distributions over the network.

## Goals

- Boot from USB
- Detect hardware
- Connect through Ethernet or Wi-Fi
- Retrieve Linux distribution manifests
- Download and verify installer resources
- Launch official Linux installers
- Protect storage devices against accidental destructive operations

## Status

Early development.

## Initial platform

- x86_64
- UEFI
- Rust
- Debian Live
- NetworkManager
- Ratatui

## Safety

This project will eventually perform destructive disk operations.

No disk operation must be executed without explicit device validation and user confirmation.

## License

To be defined.
