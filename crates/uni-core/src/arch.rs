use std::fmt;

/// CPU architecture, shared across hardware detection and catalog manifests
/// so a release can be matched against the machine it would install onto.
///
/// The MVP targets [`Architecture::X86_64`] only; the other variants exist
/// so `uni-hardware` and `uni-catalog` do not need to change shape when
/// BIOS/ARM64 support lands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Architecture {
    X86_64,
    Aarch64,
    Other,
}

impl Architecture {
    /// Architecture of the machine currently running this process.
    pub fn current() -> Self {
        Self::from(std::env::consts::ARCH)
    }
}

impl From<&str> for Architecture {
    fn from(value: &str) -> Self {
        match value {
            "x86_64" => Architecture::X86_64,
            "aarch64" => Architecture::Aarch64,
            _ => Architecture::Other,
        }
    }
}

impl fmt::Display for Architecture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Architecture::X86_64 => "x86_64",
            Architecture::Aarch64 => "aarch64",
            Architecture::Other => "unknown",
        };
        f.write_str(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_known_architectures() {
        assert_eq!(Architecture::from("x86_64"), Architecture::X86_64);
        assert_eq!(Architecture::from("aarch64"), Architecture::Aarch64);
        assert_eq!(Architecture::from("riscv64"), Architecture::Other);
    }

    #[test]
    fn round_trips_through_display() {
        assert_eq!(Architecture::X86_64.to_string(), "x86_64");
    }
}
