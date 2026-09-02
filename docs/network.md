# Networking

## Two data sources, deliberately not merged

`uni-network` gets its data from two independent sources, chosen for what
each is actually good at:

- **Interface listing** (`detect_interfaces`) reads `/sys/class/net`
  directly. No daemon round trip, works even if `NetworkManager` hasn't
  finished settling after boot, and cheap enough to call on every TUI
  refresh. A device is classified `WiFi` if it has a `wireless/`
  subdirectory, `Ethernet` if its `type` file reads `1` (`ARPHRD_ETHER`),
  `Loopback` for `lo`, `Other` otherwise.
- **Everything that requires a network stack decision** (scanning,
  connecting, disconnecting, checking connectivity) goes through the
  `NetworkBackend` trait, implemented today by `NetworkManagerBackend` on
  top of `nmcli -t` (terse, script-friendly output).

```rust
pub trait NetworkBackend {
    fn interfaces(&self) -> Result<Vec<Interface>>;
    fn scan_wifi(&self, interface: &str) -> Result<Vec<WifiNetwork>>;
    fn connect_wifi(&self, interface: &str, ssid: &str, password: Option<&str>) -> Result<()>;
    fn disconnect(&self, interface: &str) -> Result<()>;
    fn connectivity(&self) -> Result<ConnectivityState>;
}
```

Swapping `NetworkManagerBackend` for a native D-Bus implementation later
only touches `uni-network/src/backend.rs` — nothing that depends on the
trait needs to change.

## WPA/WPA2/WPA3 is never reimplemented

`connect_wifi` shells out to
`nmcli device wifi connect <ssid> password <secret> ifname <iface>`.
NetworkManager owns the entire handshake; this crate never touches a
raw `wpa_supplicant` config or implements any part of the WPA protocol
itself.

## Passwords never reach the logs

`uni_core::process::run` logs a command's full argument list at `debug`.
`connect_wifi` uses `run_redacted` instead whenever a password is present
— it logs only the command name (`nmcli`), never the argument list where
the password lives. This is the one place in the workspace a secret is
handled as input; see [`security.md`](security.md#secrets-never-reach-the-logs).

## Parsing `nmcli -t` output

`nmcli`'s terse mode (`-t -f FIELD,FIELD,...`) is colon-separated with
literal colons in a field escaped as `\:` (an SSID can legitimately
contain a colon). `uni_network::wifi::split_nmcli_fields` is a small,
pure, unit-tested tokenizer built specifically for that escaping rule,
used by both the Wi-Fi scan parser and (indirectly, via the same
convention) anything else that reads `nmcli -t` output.

`nmcli networking connectivity check` returns one of `full`, `limited`,
`portal`, `none`, `unknown`; `parse_connectivity` maps those to
`ConnectivityState::{Online, Limited, Offline, Unknown}` (`limited` and
`portal` both mean "some network below application-level Internet
access," which is what the TUI needs to know, not the distinction between
them).

## Status in phase 1

Interface detection is wired into `uni_hardware::HardwareSnapshot` and
shown in the TUI today. Wi-Fi scanning/connecting and the connectivity
check are implemented and unit-tested (on their pure parsing logic) but
not yet called from any screen — see `docs/roadmap.md` phase 5.
