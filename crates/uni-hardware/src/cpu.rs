//! CPU detection via `/proc/cpuinfo`.

use std::collections::HashSet;
use std::fs;
use std::path::Path;

use crate::error::{HardwareError, Result};

#[derive(Debug, Clone, serde::Serialize)]
pub struct CpuInfo {
    pub architecture: String,
    pub model: String,
    pub physical_cores: usize,
    pub logical_cores: usize,
}

pub fn detect_cpu() -> Result<CpuInfo> {
    detect_cpu_at(Path::new("/proc/cpuinfo"))
}

pub fn detect_cpu_at(path: &Path) -> Result<CpuInfo> {
    let raw = fs::read_to_string(path).map_err(|source| {
        HardwareError::Core(uni_core::CoreError::Io {
            path: path.to_path_buf(),
            source,
        })
    })?;
    Ok(parse_cpuinfo(
        &raw,
        uni_core::Architecture::current().to_string(),
    ))
}

/// Pure parser for `/proc/cpuinfo` content. `physical_cores` falls back to
/// `logical_cores` on architectures (e.g. some ARM kernels) that omit the
/// `physical id`/`core id` fields.
pub fn parse_cpuinfo(raw: &str, architecture: String) -> CpuInfo {
    let mut model: Option<String> = None;
    let mut logical_cores = 0usize;
    let mut unique_cores: HashSet<(String, String)> = HashSet::new();
    let mut current_physical: Option<String> = None;
    let mut current_core: Option<String> = None;

    let flush = |physical: &mut Option<String>,
                 core: &mut Option<String>,
                 set: &mut HashSet<(String, String)>| {
        if let (Some(p), Some(c)) = (physical.take(), core.take()) {
            set.insert((p, c));
        }
    };

    for line in raw.lines() {
        if line.trim().is_empty() {
            flush(&mut current_physical, &mut current_core, &mut unique_cores);
            continue;
        }
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim().to_string();
        match key {
            "model name" | "Model" | "cpu model" => {
                model.get_or_insert(value);
            }
            "processor" => {
                logical_cores += 1;
                continue;
            }
            "physical id" => {
                current_physical = Some(value);
                continue;
            }
            "core id" => {
                current_core = Some(value);
                continue;
            }
            _ => continue,
        };
    }
    flush(&mut current_physical, &mut current_core, &mut unique_cores);

    let physical_cores = if unique_cores.is_empty() {
        logical_cores
    } else {
        unique_cores.len()
    };

    CpuInfo {
        architecture,
        model: model.unwrap_or_else(|| "unknown".to_string()),
        physical_cores: physical_cores.max(1),
        logical_cores: logical_cores.max(1),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DUAL_CORE_HT: &str = "\
processor\t: 0
model name\t: AMD Ryzen 7 5800X 8-Core Processor
physical id\t: 0
core id\t: 0

processor\t: 1
model name\t: AMD Ryzen 7 5800X 8-Core Processor
physical id\t: 0
core id\t: 0

processor\t: 2
model name\t: AMD Ryzen 7 5800X 8-Core Processor
physical id\t: 0
core id\t: 1

processor\t: 3
model name\t: AMD Ryzen 7 5800X 8-Core Processor
physical id\t: 0
core id\t: 1
";

    #[test]
    fn counts_logical_and_physical_cores_with_smt() {
        let cpu = parse_cpuinfo(DUAL_CORE_HT, "x86_64".to_string());
        assert_eq!(cpu.model, "AMD Ryzen 7 5800X 8-Core Processor");
        assert_eq!(cpu.logical_cores, 4);
        assert_eq!(cpu.physical_cores, 2);
        assert_eq!(cpu.architecture, "x86_64");
    }

    #[test]
    fn falls_back_to_logical_cores_without_physical_id() {
        let raw = "processor\t: 0\nmodel name\t: Some ARM Core\n\nprocessor\t: 1\nmodel name\t: Some ARM Core\n";
        let cpu = parse_cpuinfo(raw, "aarch64".to_string());
        assert_eq!(cpu.logical_cores, 2);
        assert_eq!(cpu.physical_cores, 2);
    }

    #[test]
    fn handles_empty_input() {
        let cpu = parse_cpuinfo("", "x86_64".to_string());
        assert_eq!(cpu.model, "unknown");
        assert_eq!(cpu.logical_cores, 1);
        assert_eq!(cpu.physical_cores, 1);
    }
}
