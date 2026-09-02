//! Network interface detection and a `NetworkManager`-backed abstraction
//! over Ethernet/Wi-Fi. See `docs/network.md`.
//!
//! Interface listing ([`detect_interfaces`]) reads `/sys/class/net`
//! directly and needs no daemon running. Everything that requires
//! NetworkManager (scanning, connecting, connectivity checks) goes through
//! the [`NetworkBackend`] trait, implemented today by
//! [`NetworkManagerBackend`].

mod backend;
mod error;
mod interfaces;
mod wifi;

pub use backend::{NetworkBackend, NetworkManagerBackend};
pub use error::{NetworkError, Result};
pub use interfaces::{Interface, InterfaceKind, detect_interfaces, parse_interfaces_from};
pub use wifi::{ConnectivityState, WifiNetwork, parse_connectivity, parse_wifi_scan};
