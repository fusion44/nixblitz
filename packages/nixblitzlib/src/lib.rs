use core::fmt;
use std::str::FromStr;

use log::warn;

pub mod app_config;
pub mod app_option_data;
pub mod apply_changes;
pub mod apps;
pub mod bitcoind;
pub mod blitz_api;
pub mod blitz_webui;
pub mod cln;
pub mod errors;
pub mod lnd;
pub mod locales;
pub mod nix_base_config;
pub mod number_value;
pub mod project;
pub mod strings;
pub mod timezones;
pub mod utils;

use strum::{EnumCount, EnumIter, IntoEnumIterator, VariantArray};

/// Represents the detected system platform and architecture.
#[derive(Debug, PartialEq, Eq, Clone, Copy, EnumIter, EnumCount, VariantArray)]
pub enum SystemPlatform {
    /// System is running on x86_64 bare metal
    X86_64BareMetal,
    /// System is running in a virtual machine on x86_64
    X86_64Vm,
    /// System is running on ARM64 (aarch64) architecture.
    Arm64,
    /// System is running on Raspberry Pi 4
    Pi4,
    /// System is running on Raspberry Pi 5
    Pi5,
    /// The specific architecture/platform is not recognized or supported
    Unsupported,
}

impl fmt::Display for SystemPlatform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SystemPlatform::X86_64BareMetal => write!(f, "x86_64 Bare Metal"),
            SystemPlatform::X86_64Vm => write!(f, "x86_64 Virtual Machine"),
            SystemPlatform::Arm64 => write!(f, "ARM64"),
            SystemPlatform::Pi4 => write!(f, "Raspberry Pi 4"),
            SystemPlatform::Pi5 => write!(f, "Raspberry Pi 5"),
            SystemPlatform::Unsupported => write!(f, "Unsupported Platform"),
        }
    }
}

impl SystemPlatform {
    /// Returns the NixOS system declaration for the platform.
    pub fn as_nixos_system_name(&self) -> &'static str {
        match self {
            SystemPlatform::X86_64BareMetal => "nixblitzx86",
            SystemPlatform::X86_64Vm => "nixblitzx86vm",
            SystemPlatform::Arm64 => "nixblitzarm64",
            SystemPlatform::Pi4 => "nixblitzpi4",
            SystemPlatform::Pi5 => "nixblitzpi5",
            SystemPlatform::Unsupported => "unsupported",
        }
    }

    /// Returns the canonical "short string" representation for the platform.
    pub fn as_short_str(&self) -> &'static str {
        match self {
            SystemPlatform::X86_64BareMetal => "x86_64_bm",
            SystemPlatform::X86_64Vm => "x86_64_vm",
            SystemPlatform::Arm64 => "arm64",
            SystemPlatform::Pi4 => "pi4",
            SystemPlatform::Pi5 => "pi5",
            SystemPlatform::Unsupported => "unsupported",
        }
    }

    /// Returns an array of the canonical "short string" representations for all platforms.
    /// The order of strings in the array corresponds to the iteration order from `SystemPlatform::iter()`.
    /// The array size is determined by `SystemPlatform::COUNT`.
    /// A unit test should verify that `all_short_strs()[i] == SystemPlatform::iter().nth(i).unwrap().as_short_str()`.
    pub fn all_short_strs() -> [&'static str; SystemPlatform::COUNT] {
        [
            "x86_64_bm",   // X86_64BareMetal.as_short_str()
            "x86_64_vm",   // X86_64Vm.as_short_str()
            "arm64",       // Arm64.as_short_str()
            "pi4",         // Pi4.as_short_str()
            "pi5",         // Pi5.as_short_str()
            "unsupported", // Unsupported.as_short_str()
        ]
    }

    /// Attempts to parse a `SystemPlatform` from its "short string" representation (case-insensitive).
    pub fn from_short_str_option(s: &str) -> Option<SystemPlatform> {
        for variant in SystemPlatform::iter() {
            if variant.as_short_str().eq_ignore_ascii_case(s) {
                return Some(variant);
            }
        }
        warn!("Unable to parse platform from string: {}", s);
        None
    }
}

impl FromStr for SystemPlatform {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for variant in SystemPlatform::iter() {
            // Use Strum's EnumIter
            if variant.as_short_str().eq_ignore_ascii_case(s) {
                return Ok(variant);
            }
        }
        let available_options = SystemPlatform::all_short_strs().join(", ");
        let err_msg = format!(
            "Failed to parse '{}' as SystemPlatform. Expected one of (case-insensitive): [{}].",
            s, available_options
        );
        warn!("{}", err_msg);
        Err(err_msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use strum::IntoEnumIterator;

    #[test]
    fn test_display_format() {
        assert_eq!(
            SystemPlatform::X86_64BareMetal.to_string(),
            "x86_64 Bare Metal"
        );
        assert_eq!(SystemPlatform::Pi5.to_string(), "Raspberry Pi 5");
    }

    #[test]
    fn test_to_nixos_system_name() {
        assert_eq!(
            SystemPlatform::X86_64BareMetal.as_nixos_system_name(),
            "nixblitzx86"
        );
        assert_eq!(
            SystemPlatform::Unsupported.as_nixos_system_name(),
            "unsupported"
        );
    }

    #[test]
    fn test_as_short_str() {
        assert_eq!(SystemPlatform::X86_64Vm.as_short_str(), "x86_64_vm");
        assert_eq!(SystemPlatform::Pi4.as_short_str(), "pi4");
    }

    #[test]
    fn test_all_short_strs_consistency_and_content() {
        let all_strs_array = SystemPlatform::all_short_strs();
        // Check length using EnumCount
        assert_eq!(
            SystemPlatform::COUNT,
            all_strs_array.len(),
            "Mismatch in length between SystemPlatform::COUNT and all_short_strs array"
        );

        // Iterate through variants using EnumIter and check against the array
        for (i, variant) in SystemPlatform::iter().enumerate() {
            assert_eq!(variant.as_short_str(), all_strs_array[i],
                       "Mismatch at index {} for variant {:?}: as_short_str() is '{}', all_short_strs()[i] is '{}'",
                       i, variant, variant.as_short_str(), all_strs_array[i]);
        }
    }

    #[test]
    fn test_from_short_str_option() {
        assert_eq!(
            SystemPlatform::from_short_str_option("x86_64_vm"),
            Some(SystemPlatform::X86_64Vm)
        );
        assert_eq!(
            SystemPlatform::from_short_str_option("arm64"),
            Some(SystemPlatform::Arm64)
        );
        assert_eq!(
            SystemPlatform::from_short_str_option("ARM64"),
            Some(SystemPlatform::Arm64)
        );
        assert_eq!(
            SystemPlatform::from_short_str_option("pi5"),
            Some(SystemPlatform::Pi5)
        );
        assert_eq!(
            SystemPlatform::from_short_str_option("unknown_platform"),
            None
        );
    }

    #[test]
    fn test_from_str_trait() {
        assert_eq!("Pi4".parse::<SystemPlatform>(), Ok(SystemPlatform::Pi4));
        assert_eq!("pi4".parse::<SystemPlatform>(), Ok(SystemPlatform::Pi4));
        assert_eq!(
            "X86_64_vm".parse::<SystemPlatform>(),
            Ok(SystemPlatform::X86_64Vm)
        );

        let parse_result = "invalid".parse::<SystemPlatform>();
        assert!(parse_result.is_err());
        if let Err(e) = parse_result {
            assert!(e.contains("Failed to parse 'invalid' as SystemPlatform"));
        }
    }

    #[test]
    fn test_enum_iter_and_count() {
        assert_eq!(SystemPlatform::iter().count(), SystemPlatform::COUNT);
        assert_eq!(SystemPlatform::COUNT, 6); // Explicitly check count if it's stable
        let mut found_pi4 = false;
        for platform in SystemPlatform::iter() {
            if platform == SystemPlatform::Pi4 {
                found_pi4 = true;
            }
        }
        assert!(found_pi4, "EnumIter did not yield Pi4 variant");
    }
}
