//! [`HardwareSnapshot`]: the single read-only "what is this machine"
//! aggregate the TUI renders. Composes `uni-storage` and `uni-network`
//! rather than re-implementing disk/interface parsing here — see
//! `docs/architecture.md` for why `uni-hardware` is a facade over the
//! domain crates instead of a third parser for the same data.

use crate::boot_mode::{BootMode, detect_boot_mode};
use crate::cpu::{CpuInfo, detect_cpu};
use crate::error::Result;
use crate::gpu::{GpuInfo, detect_gpus};
use crate::memory::{MemoryInfo, detect_memory};

#[derive(Debug, Clone, serde::Serialize)]
pub struct HardwareSnapshot {
    pub cpu: CpuInfo,
    pub memory: MemoryInfo,
    pub gpus: Vec<GpuInfo>,
    pub boot_mode: BootMode,
    pub disks: Vec<uni_storage::DiskInfo>,
    pub interfaces: Vec<uni_network::Interface>,
}

/// Builds a full [`HardwareSnapshot`].
///
/// CPU, RAM, disks and network interfaces are load-bearing for the MVP and
/// fail the whole snapshot if they can't be read. GPU detection
/// (`lspci`) is best-effort: a minimal live environment without
/// `pciutils` still gets a usable snapshot, just with an empty GPU list.
pub fn detect() -> Result<HardwareSnapshot> {
    let gpus = detect_gpus().unwrap_or_else(|err| {
        tracing::warn!(%err, "GPU detection failed, continuing without it");
        Vec::new()
    });

    Ok(HardwareSnapshot {
        cpu: detect_cpu()?,
        memory: detect_memory()?,
        gpus,
        boot_mode: detect_boot_mode(),
        disks: uni_storage::detect_disks()?,
        interfaces: uni_network::detect_interfaces()?,
    })
}
