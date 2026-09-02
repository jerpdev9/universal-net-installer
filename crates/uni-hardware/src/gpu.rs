//! GPU detection via `lspci -mm` (the stable, quote-delimited machine
//! format `pciutils` documents, instead of scraping the human-readable
//! table).

use crate::error::Result;

#[derive(Debug, Clone, serde::Serialize)]
pub struct GpuInfo {
    pub vendor: String,
    pub model: String,
}

/// GPU detection is best-effort: a minimal live/container environment may
/// not ship `pciutils`. Callers should treat an error here as "unknown",
/// not fatal to the rest of the hardware snapshot.
pub fn detect_gpus() -> Result<Vec<GpuInfo>> {
    let raw = uni_core::process::run("lspci", &["-mm"])?;
    Ok(parse_lspci_mm(&raw))
}

/// Pure parser for `lspci -mm` output.
pub fn parse_lspci_mm(raw: &str) -> Vec<GpuInfo> {
    raw.lines()
        .filter_map(|line| {
            let fields = quoted_fields(line);
            let class = fields.first()?;
            let is_display_device = [
                "VGA compatible controller",
                "3D controller",
                "Display controller",
            ]
            .iter()
            .any(|needle| class.contains(needle));
            if !is_display_device {
                return None;
            }
            Some(GpuInfo {
                vendor: fields
                    .get(1)
                    .cloned()
                    .unwrap_or_else(|| "unknown".to_string()),
                model: fields
                    .get(2)
                    .cloned()
                    .unwrap_or_else(|| "unknown".to_string()),
            })
        })
        .collect()
}

/// Extracts every `"..."`-quoted field from an `lspci -mm` line, in order.
fn quoted_fields(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for c in line.chars() {
        match c {
            '"' => {
                if in_quotes {
                    fields.push(std::mem::take(&mut current));
                }
                in_quotes = !in_quotes;
            }
            _ if in_quotes => current.push(c),
            _ => {}
        }
    }
    fields
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"00:02.0 "VGA compatible controller" "Intel Corporation" "AlderLake-P Integrated Graphics Controller" -ra1 "Dell" "Device 0000"
01:00.0 "3D controller" "NVIDIA Corporation" "GA104 [GeForce RTX 3070]" -ra1 "NVIDIA Corporation" "GA104 [GeForce RTX 3070]"
00:1f.3 "Audio device" "Intel Corporation" "Some Audio Controller"
"#;

    #[test]
    fn extracts_only_display_controllers() {
        let gpus = parse_lspci_mm(SAMPLE);
        assert_eq!(gpus.len(), 2);
        assert_eq!(gpus[0].vendor, "Intel Corporation");
        assert!(gpus[0].model.contains("Integrated Graphics"));
        assert_eq!(gpus[1].vendor, "NVIDIA Corporation");
    }

    #[test]
    fn ignores_non_display_lines() {
        let gpus = parse_lspci_mm("00:1f.3 \"Audio device\" \"Intel Corporation\" \"X\"\n");
        assert!(gpus.is_empty());
    }
}
