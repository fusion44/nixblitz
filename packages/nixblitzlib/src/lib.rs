use strum::Display;

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

/// Represents the detected system platform and architecture.
#[derive(Debug, Display, PartialEq, Eq, Clone, Copy)]
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

impl SystemPlatform {
    /// Returns the NixOS system declaration for the given platform as defined in the
    /// flake.nix file of the template.
    pub fn to_nixos_system_name(&self) -> &'static str {
        match self {
            SystemPlatform::X86_64BareMetal => "nixblitzx86",
            SystemPlatform::X86_64Vm => "nixblitzx86vm",
            SystemPlatform::Arm64 => "nixblitzarm64",
            SystemPlatform::Pi4 => "nixblitzpi4",
            SystemPlatform::Pi5 => "nixblitzpi5",
            SystemPlatform::Unsupported => "unsupported",
        }
    }

    /// Returns an array of string representations of all supported system platforms.
    pub fn to_string_array() -> [&'static str; 6] {
        [
            "x86_64 bare metal",
            "x86_64 vm",
            "arm64",
            "Pi4",
            "Pi5",
            "unsupported",
        ]
    }

    pub fn from_string(s: &str) -> Option<SystemPlatform> {
        match s {
            "x86_64 bare metal" => Some(SystemPlatform::X86_64BareMetal),
            "x86_64 vm" => Some(SystemPlatform::X86_64Vm),
            "arm64" => Some(SystemPlatform::Arm64),
            "Pi4" => Some(SystemPlatform::Pi4),
            "Pi5" => Some(SystemPlatform::Pi5),
            "unsupported" => Some(SystemPlatform::Unsupported),
            _ => None,
        }
    }
}
