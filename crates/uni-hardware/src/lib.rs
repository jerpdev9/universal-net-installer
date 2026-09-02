//! Aggregates CPU, RAM, GPU, firmware boot mode, disk and network facts
//! into one [`HardwareSnapshot`] for the TUI to render.
//!
//! This crate does not parse `lsblk`/`/sys/class/net` itself: it composes
//! [`uni_storage`] and [`uni_network`], which own that logic. See
//! `docs/architecture.md`.

mod boot_mode;
mod cpu;
mod error;
mod gpu;
mod memory;
mod snapshot;

pub use boot_mode::{BootMode, detect_boot_mode, detect_boot_mode_at};
pub use cpu::{CpuInfo, detect_cpu, parse_cpuinfo};
pub use error::{HardwareError, Result};
pub use gpu::{GpuInfo, detect_gpus, parse_lspci_mm};
pub use memory::{MemoryInfo, detect_memory, parse_meminfo};
pub use snapshot::{HardwareSnapshot, detect};
