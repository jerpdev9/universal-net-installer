//! Application state.
//!
//! Phase 1 only rendered a [`uni_hardware::HardwareSnapshot`]. This phase
//! adds the Wi-Fi connection flow (roadmap phase 5): scanning, picking a
//! network, entering a password and connecting, all through
//! [`uni_network::NetworkBackend`] — no WPA handshake logic lives here,
//! only UI state. OS selection, download and install state still don't
//! exist — that's the catalog/downloader/installer phases.

use std::time::Instant;

use uni_hardware::HardwareSnapshot;
use uni_network::{
    ConnectivityState, InterfaceKind, NetworkBackend, NetworkManagerBackend, WifiNetwork,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Dashboard,
    WifiList,
    WifiPassword,
}

pub struct StatusMessage {
    pub text: String,
    pub is_error: bool,
}

pub struct App {
    pub snapshot: Option<HardwareSnapshot>,
    pub error: Option<String>,
    pub last_refreshed: Option<Instant>,
    pub should_quit: bool,

    pub screen: Screen,
    pub connectivity: Option<ConnectivityState>,
    pub status: Option<StatusMessage>,

    pub wifi_interface: Option<String>,
    pub wifi_networks: Vec<WifiNetwork>,
    pub wifi_selected: usize,
    pub wifi_pending_ssid: Option<String>,
    pub password_input: String,

    network: NetworkManagerBackend,
}

impl App {
    pub fn new() -> Self {
        Self {
            snapshot: None,
            error: None,
            last_refreshed: None,
            should_quit: false,
            screen: Screen::Dashboard,
            connectivity: None,
            status: None,
            wifi_interface: None,
            wifi_networks: Vec::new(),
            wifi_selected: 0,
            wifi_pending_ssid: None,
            password_input: String::new(),
            network: NetworkManagerBackend::new(),
        }
    }

    /// Re-runs hardware detection and the connectivity check. Both are a
    /// handful of fast reads (`/proc`, `/sys`) and short-lived commands
    /// (`lsblk`, `lspci`, `nmcli`), so running them synchronously on
    /// key-press keeps the TUI simple without a noticeable stall.
    pub fn refresh(&mut self) {
        match uni_hardware::detect() {
            Ok(snapshot) => {
                self.snapshot = Some(snapshot);
                self.error = None;
            }
            Err(err) => {
                self.error = Some(err.to_string());
            }
        }
        self.last_refreshed = Some(Instant::now());
        self.connectivity = self.network.connectivity().ok();
    }

    /// Picks the first Wi-Fi-capable interface out of the last detected
    /// snapshot, scans it, and switches to the network list screen.
    pub fn open_wifi_screen(&mut self) {
        let Some(iface) = self.wifi_interface_name() else {
            self.set_status("no Wi-Fi interface detected", true);
            return;
        };
        self.wifi_interface = Some(iface.clone());

        match self.network.scan_wifi(&iface) {
            Ok(mut networks) => {
                networks.sort_by_key(|n| std::cmp::Reverse(n.signal));
                self.wifi_networks = networks;
                self.wifi_selected = 0;
                self.screen = Screen::WifiList;
                self.status = None;
            }
            Err(err) => self.set_status(&format!("Wi-Fi scan failed: {err}"), true),
        }
    }

    pub fn wifi_move_selection(&mut self, delta: isize) {
        if self.wifi_networks.is_empty() {
            return;
        }
        let len = self.wifi_networks.len() as isize;
        let next = (self.wifi_selected as isize + delta).rem_euclid(len);
        self.wifi_selected = next as usize;
    }

    /// Enter selects the highlighted network: open networks connect
    /// immediately, secured ones move to the password screen.
    pub fn wifi_confirm_selection(&mut self) {
        let Some(network) = self.wifi_networks.get(self.wifi_selected) else {
            return;
        };
        if is_open_security(&network.security) {
            let ssid = network.ssid.clone();
            self.connect_wifi(&ssid, None);
        } else {
            self.wifi_pending_ssid = Some(network.ssid.clone());
            self.password_input.clear();
            self.screen = Screen::WifiPassword;
        }
    }

    pub fn password_push_char(&mut self, c: char) {
        self.password_input.push(c);
    }

    pub fn password_pop_char(&mut self) {
        self.password_input.pop();
    }

    pub fn wifi_submit_password(&mut self) {
        let Some(ssid) = self.wifi_pending_ssid.clone() else {
            self.screen = Screen::Dashboard;
            return;
        };
        let password = std::mem::take(&mut self.password_input);
        self.connect_wifi(&ssid, Some(&password));
    }

    pub fn cancel_wifi_flow(&mut self) {
        self.wifi_pending_ssid = None;
        self.password_input.clear();
        self.screen = Screen::Dashboard;
    }

    fn connect_wifi(&mut self, ssid: &str, password: Option<&str>) {
        let Some(iface) = self.wifi_interface.clone() else {
            self.set_status("no Wi-Fi interface detected", true);
            return;
        };
        match self.network.connect_wifi(&iface, ssid, password) {
            Ok(()) => {
                self.set_status(&format!("connected to {ssid}"), false);
                self.wifi_pending_ssid = None;
                self.password_input.clear();
                self.screen = Screen::Dashboard;
                self.refresh();
            }
            Err(err) => self.set_status(&format!("failed to connect to {ssid}: {err}"), true),
        }
    }

    fn wifi_interface_name(&self) -> Option<String> {
        self.snapshot
            .as_ref()?
            .interfaces
            .iter()
            .find(|i| i.kind == InterfaceKind::WiFi)
            .map(|i| i.name.clone())
    }

    fn set_status(&mut self, text: &str, is_error: bool) {
        self.status = Some(StatusMessage {
            text: text.to_string(),
            is_error,
        });
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

/// `nmcli`'s security column reads `--` for an open network.
fn is_open_security(security: &str) -> bool {
    let s = security.trim();
    s.is_empty() || s == "--"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_open_security_recognizes_nmclis_dash_dash() {
        assert!(is_open_security("--"));
        assert!(is_open_security(""));
        assert!(!is_open_security("WPA2"));
    }

    #[test]
    fn wifi_move_selection_wraps_around() {
        let mut app = App::new();
        app.wifi_networks = vec![
            WifiNetwork {
                ssid: "a".to_string(),
                signal: 80,
                security: "WPA2".to_string(),
                in_use: false,
            },
            WifiNetwork {
                ssid: "b".to_string(),
                signal: 60,
                security: "--".to_string(),
                in_use: false,
            },
        ];
        app.wifi_selected = 0;
        app.wifi_move_selection(-1);
        assert_eq!(app.wifi_selected, 1);
        app.wifi_move_selection(1);
        assert_eq!(app.wifi_selected, 0);
    }

    #[test]
    fn password_input_pushes_and_pops() {
        let mut app = App::new();
        app.password_push_char('h');
        app.password_push_char('i');
        assert_eq!(app.password_input, "hi");
        app.password_pop_char();
        assert_eq!(app.password_input, "h");
    }
}
