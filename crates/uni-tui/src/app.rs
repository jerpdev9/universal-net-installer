//! Application state.
//!
//! Phase 1 only rendered a [`uni_hardware::HardwareSnapshot`]. Phase 5
//! added the Wi-Fi connection flow: scanning, picking a network, entering
//! a password and connecting, all through [`uni_network::NetworkBackend`]
//! — no WPA handshake logic lives here, only UI state. This phase (6)
//! adds browsing the distribution catalog: loading `manifests/*.yaml` via
//! `uni-catalog`, picking a distribution, then a release. Picking a
//! release only records the choice in a status message — nothing
//! downloads yet, that's phase 7.

use std::path::PathBuf;
use std::time::Instant;

use uni_catalog::Manifest;
use uni_hardware::HardwareSnapshot;
use uni_network::{
    ConnectivityState, InterfaceKind, NetworkBackend, NetworkManagerBackend, WifiNetwork,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Dashboard,
    WifiList,
    WifiPassword,
    DistroList,
    ReleaseList,
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

    pub catalog: Vec<Manifest>,
    pub catalog_error: Option<String>,
    pub distro_selected: usize,
    pub release_selected: usize,

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
            catalog: Vec::new(),
            catalog_error: None,
            distro_selected: 0,
            release_selected: 0,
            network: NetworkManagerBackend::new(),
        }
    }

    /// Loads `manifests/*.yaml` via `uni_catalog::load_catalog_dir`. Safe
    /// to call more than once (e.g. to pick up manifest edits) — it just
    /// replaces `catalog`.
    pub fn load_catalog(&mut self) {
        match resolve_manifests_dir() {
            Some(dir) => self.load_catalog_from(&dir),
            None => {
                self.catalog_error = Some("could not locate the manifests/ directory".to_string());
            }
        }
    }

    fn load_catalog_from(&mut self, dir: &std::path::Path) {
        match uni_catalog::load_catalog_dir(dir) {
            Ok(manifests) => {
                self.catalog = manifests;
                self.catalog_error = None;
            }
            Err(err) => self.catalog_error = Some(err.to_string()),
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
        if let Some(next) = wrap_index(self.wifi_selected, delta, self.wifi_networks.len()) {
            self.wifi_selected = next;
        }
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

    /// Switches to the distribution list. `load_catalog` is expected to
    /// have already run (at startup, in `main`); this only surfaces
    /// whatever it found.
    pub fn open_distro_screen(&mut self) {
        if let Some(err) = &self.catalog_error {
            self.set_status(&format!("catalog failed to load: {err}"), true);
            return;
        }
        if self.catalog.is_empty() {
            self.set_status("catalog is empty", true);
            return;
        }
        self.distro_selected = 0;
        self.screen = Screen::DistroList;
        self.status = None;
    }

    pub fn distro_move_selection(&mut self, delta: isize) {
        if let Some(next) = wrap_index(self.distro_selected, delta, self.catalog.len()) {
            self.distro_selected = next;
        }
    }

    /// Enter on a distribution moves to its release list.
    pub fn distro_confirm_selection(&mut self) {
        if self.catalog.get(self.distro_selected).is_none() {
            return;
        }
        self.release_selected = 0;
        self.screen = Screen::ReleaseList;
    }

    pub fn release_move_selection(&mut self, delta: isize) {
        let Some(manifest) = self.catalog.get(self.distro_selected) else {
            return;
        };
        if let Some(next) = wrap_index(self.release_selected, delta, manifest.releases.len()) {
            self.release_selected = next;
        }
    }

    /// Enter on a release just records the choice in the status line.
    /// Nothing is downloaded — `uni-downloader` isn't wired in yet, see
    /// `docs/roadmap.md` phase 7.
    pub fn release_confirm_selection(&mut self) {
        let Some(manifest) = self.catalog.get(self.distro_selected) else {
            return;
        };
        let Some(release) = manifest.releases.get(self.release_selected) else {
            return;
        };
        self.set_status(
            &format!(
                "selected {} {} — download not implemented yet (docs/roadmap.md phase 7)",
                manifest.name, release.version
            ),
            false,
        );
        self.screen = Screen::Dashboard;
    }

    pub fn back_to_distro_list(&mut self) {
        self.screen = Screen::DistroList;
    }

    pub fn cancel_distro_flow(&mut self) {
        self.screen = Screen::Dashboard;
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

/// Moves `current` by `delta`, wrapping within `[0, len)`. `None` if
/// `len` is `0` (nothing to select).
fn wrap_index(current: usize, delta: isize, len: usize) -> Option<usize> {
    if len == 0 {
        return None;
    }
    let next = (current as isize + delta).rem_euclid(len as isize);
    Some(next as usize)
}

/// Locates the `manifests/` directory at runtime: an explicit
/// `UNI_MANIFESTS_DIR` override, then `./manifests` (the layout when
/// running via `cargo run` from the repository root), then a few
/// locations relative to the running executable (for a future packaged
/// deployment — see `docs/roadmap.md` phase 10).
fn resolve_manifests_dir() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("UNI_MANIFESTS_DIR") {
        let path = PathBuf::from(dir);
        if path.is_dir() {
            return Some(path);
        }
    }

    let cwd_candidate = PathBuf::from("manifests");
    if cwd_candidate.is_dir() {
        return Some(cwd_candidate);
    }

    let exe_dir = std::env::current_exe().ok()?.parent()?.to_path_buf();
    [
        exe_dir.join("manifests"),
        exe_dir.join("../manifests"),
        exe_dir.join("../../manifests"),
    ]
    .into_iter()
    .find(|candidate| candidate.is_dir())
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

    #[test]
    fn wrap_index_wraps_in_both_directions() {
        assert_eq!(wrap_index(0, -1, 3), Some(2));
        assert_eq!(wrap_index(2, 1, 3), Some(0));
        assert_eq!(wrap_index(1, 1, 3), Some(2));
    }

    #[test]
    fn wrap_index_returns_none_for_empty_list() {
        assert_eq!(wrap_index(0, 1, 0), None);
    }

    fn manifests_dir() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../manifests")
    }

    #[test]
    fn load_catalog_from_populates_catalog() {
        let mut app = App::new();
        app.load_catalog_from(&manifests_dir());
        assert!(app.catalog_error.is_none());
        assert_eq!(app.catalog.len(), 4);
    }

    #[test]
    fn open_distro_screen_reports_catalog_error() {
        let mut app = App::new();
        app.load_catalog_from(std::path::Path::new("/nonexistent/manifests"));
        app.open_distro_screen();
        assert_eq!(app.screen, Screen::Dashboard);
        assert!(app.status.is_some());
        assert!(app.status.unwrap().is_error);
    }

    #[test]
    fn distro_and_release_navigation_selects_a_release() {
        let mut app = App::new();
        app.load_catalog_from(&manifests_dir());
        app.open_distro_screen();
        assert_eq!(app.screen, Screen::DistroList);

        app.distro_confirm_selection();
        assert_eq!(app.screen, Screen::ReleaseList);

        app.release_confirm_selection();
        assert_eq!(app.screen, Screen::Dashboard);
        let status = app.status.unwrap();
        assert!(!status.is_error);
        assert!(status.text.contains("download not implemented yet"));
    }
}
