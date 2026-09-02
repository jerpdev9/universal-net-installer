//! [`NetworkBackend`]: the abstraction the rest of the workspace codes
//! against instead of calling `nmcli` directly. `NetworkManagerBackend` is
//! today's implementation; swapping to native D-Bus calls to
//! NetworkManager later only touches this file.

use crate::error::Result;
use crate::interfaces::{Interface, detect_interfaces};
use crate::wifi::{ConnectivityState, WifiNetwork, parse_connectivity, parse_wifi_scan};

pub trait NetworkBackend {
    fn interfaces(&self) -> Result<Vec<Interface>>;
    fn scan_wifi(&self, interface: &str) -> Result<Vec<WifiNetwork>>;
    fn connect_wifi(&self, interface: &str, ssid: &str, password: Option<&str>) -> Result<()>;
    fn disconnect(&self, interface: &str) -> Result<()>;
    fn connectivity(&self) -> Result<ConnectivityState>;
}

/// `NetworkBackend` implemented on top of `NetworkManager`'s `nmcli` CLI.
///
/// WPA/WPA2/WPA3 handshakes are never implemented here: NetworkManager
/// owns that entirely. This type only shells out to `nmcli` and parses its
/// terse (`-t`) output.
#[derive(Debug, Default)]
pub struct NetworkManagerBackend;

impl NetworkManagerBackend {
    pub fn new() -> Self {
        Self
    }
}

impl NetworkBackend for NetworkManagerBackend {
    fn interfaces(&self) -> Result<Vec<Interface>> {
        detect_interfaces()
    }

    fn scan_wifi(&self, interface: &str) -> Result<Vec<WifiNetwork>> {
        let output = uni_core::process::run(
            "nmcli",
            &[
                "-t",
                "-f",
                "SSID,SIGNAL,SECURITY,IN-USE",
                "device",
                "wifi",
                "list",
                "ifname",
                interface,
            ],
        )?;
        Ok(parse_wifi_scan(&output))
    }

    fn connect_wifi(&self, interface: &str, ssid: &str, password: Option<&str>) -> Result<()> {
        // Passwords must never reach the logs: use `run_redacted` whenever
        // a secret is part of the argument list.
        match password {
            Some(password) => {
                uni_core::process::run_redacted(
                    "nmcli",
                    &[
                        "device", "wifi", "connect", ssid, "password", password, "ifname",
                        interface,
                    ],
                )?;
            }
            None => {
                uni_core::process::run(
                    "nmcli",
                    &["device", "wifi", "connect", ssid, "ifname", interface],
                )?;
            }
        }
        Ok(())
    }

    fn disconnect(&self, interface: &str) -> Result<()> {
        uni_core::process::run("nmcli", &["device", "disconnect", interface])?;
        Ok(())
    }

    fn connectivity(&self) -> Result<ConnectivityState> {
        let output = uni_core::process::run("nmcli", &["networking", "connectivity", "check"])?;
        Ok(parse_connectivity(&output))
    }
}
