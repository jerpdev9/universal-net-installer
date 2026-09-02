//! Network interface enumeration via `/sys/class/net`.
//!
//! This is deliberately filesystem-only (no NetworkManager/D-Bus round
//! trip) so listing interfaces works even before `NetworkManager` has
//! settled, and so it stays cheap to call on every TUI refresh.

use std::fs;
use std::path::Path;

use crate::error::Result;

const SYS_CLASS_NET: &str = "/sys/class/net";

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum InterfaceKind {
    Ethernet,
    WiFi,
    Loopback,
    Other,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Interface {
    pub name: String,
    pub kind: InterfaceKind,
    pub mac: Option<String>,
    pub is_up: bool,
    pub carrier: bool,
}

/// Lists every interface known to the kernel.
pub fn detect_interfaces() -> Result<Vec<Interface>> {
    parse_interfaces_from(Path::new(SYS_CLASS_NET))
}

/// Pure(ish) implementation reading from an arbitrary root, so tests can
/// point it at a temporary directory that mimics `/sys/class/net`.
pub fn parse_interfaces_from(root: &Path) -> Result<Vec<Interface>> {
    let mut interfaces = Vec::new();

    let entries = match fs::read_dir(root) {
        Ok(entries) => entries,
        Err(_) => return Ok(interfaces), // no /sys/class/net (e.g. dev container): report none
    };

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        let iface_dir = entry.path();
        interfaces.push(parse_one(&name, &iface_dir));
    }

    interfaces.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(interfaces)
}

fn parse_one(name: &str, iface_dir: &Path) -> Interface {
    let kind = if name == "lo" {
        InterfaceKind::Loopback
    } else if iface_dir.join("wireless").is_dir() {
        InterfaceKind::WiFi
    } else if read_trimmed(&iface_dir.join("type")).as_deref() == Some("1") {
        InterfaceKind::Ethernet
    } else {
        InterfaceKind::Other
    };

    let mac = read_trimmed(&iface_dir.join("address")).filter(|m| m != "00:00:00:00:00:00");
    let is_up = read_trimmed(&iface_dir.join("operstate")).as_deref() == Some("up");
    let carrier = read_trimmed(&iface_dir.join("carrier")).as_deref() == Some("1");

    Interface {
        name: name.to_string(),
        kind,
        mac,
        is_up,
        carrier,
    }
}

fn read_trimmed(path: &Path) -> Option<String> {
    fs::read_to_string(path).ok().map(|s| s.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn write_iface(root: &Path, name: &str, files: &[(&str, &str)]) {
        let dir = root.join(name);
        fs::create_dir_all(&dir).unwrap();
        for (file, content) in files {
            fs::write(dir.join(file), content).unwrap();
        }
    }

    #[test]
    fn detects_ethernet_wifi_and_loopback() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        write_iface(
            root,
            "eth0",
            &[
                ("type", "1"),
                ("operstate", "up"),
                ("carrier", "1"),
                ("address", "aa:bb:cc:dd:ee:ff"),
            ],
        );
        write_iface(root, "wlan0", &[("operstate", "down"), ("carrier", "0")]);
        fs::create_dir_all(root.join("wlan0/wireless")).unwrap();
        write_iface(root, "lo", &[("type", "772"), ("operstate", "unknown")]);

        let mut ifaces = parse_interfaces_from(root).unwrap();
        ifaces.sort_by(|a, b| a.name.cmp(&b.name));

        assert_eq!(ifaces.len(), 3);

        let eth0 = ifaces.iter().find(|i| i.name == "eth0").unwrap();
        assert_eq!(eth0.kind, InterfaceKind::Ethernet);
        assert!(eth0.is_up);
        assert!(eth0.carrier);
        assert_eq!(eth0.mac.as_deref(), Some("aa:bb:cc:dd:ee:ff"));

        let wlan0 = ifaces.iter().find(|i| i.name == "wlan0").unwrap();
        assert_eq!(wlan0.kind, InterfaceKind::WiFi);
        assert!(!wlan0.is_up);

        let lo = ifaces.iter().find(|i| i.name == "lo").unwrap();
        assert_eq!(lo.kind, InterfaceKind::Loopback);
    }

    #[test]
    fn missing_sys_class_net_returns_empty() {
        let ifaces = parse_interfaces_from(Path::new("/nonexistent/path")).unwrap();
        assert!(ifaces.is_empty());
    }
}
