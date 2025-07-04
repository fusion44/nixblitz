use core::fmt;

use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Hash, PartialEq, Eq, Serialize, Deserialize, Clone, Copy)]
pub enum SupportedApps {
    #[default]
    NixOS,
    BitcoinCore,
    CoreLightning,
    LND,
    BlitzAPI,
    WebUI,
}

impl SupportedApps {
    /// A constant, ordered array of all enum variants. The order here defines the cycle.
    const VARIANTS: [Self; 6] = [
        Self::NixOS,
        Self::BitcoinCore,
        Self::CoreLightning,
        Self::LND,
        Self::BlitzAPI,
        Self::WebUI,
    ];

    /// The display names corresponding to the VARIANTS array.
    const APP_NAMES: [&'static str; 6] = [
        "Nix OS",
        "Bitcoin Core",
        "Core Lightning",
        "LND",
        "Blitz Api",
        "Web UI",
    ];

    /// Returns the next app in the cycle. Wraps around at the end.
    pub fn next(&self) -> Self {
        let current_index = self.as_index();
        let next_index = (current_index + 1) % Self::VARIANTS.len();
        Self::VARIANTS[next_index]
    }

    /// Returns the previous app in the cycle. Wraps around at the beginning.
    pub fn previous(&self) -> Self {
        let current_index = self.as_index();
        let prev_index = (current_index + Self::VARIANTS.len() - 1) % Self::VARIANTS.len();
        Self::VARIANTS[prev_index]
    }

    /// Converts a string slice to a `SupportedApps` variant.
    pub fn from(s: &str) -> Option<Self> {
        let index = Self::APP_NAMES.iter().position(|&name| name == s)?;
        Self::from_id(index)
    }

    /// Converts a numeric ID to a `SupportedApps` variant.
    pub fn from_id(id: usize) -> Option<Self> {
        // .get() safely handles out-of-bounds access by returning None
        // .copied() converts Option<&Self> to Option<Self>
        Self::VARIANTS.get(id).copied()
    }

    /// Returns the string representation of the app.
    pub fn to_string(&self) -> &'static str {
        Self::APP_NAMES[self.as_index()]
    }

    /// Returns the full list of app names.
    pub fn as_string_list() -> &'static [&'static str] {
        &Self::APP_NAMES
    }

    /// Get the index of the current variant.
    pub fn as_index(&self) -> usize {
        Self::VARIANTS.iter().position(|&v| v == *self).unwrap()
    }
}

impl fmt::Display for SupportedApps {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_round_trip() {
        for app in SupportedApps::VARIANTS {
            let string = app.to_string();
            assert_eq!(SupportedApps::from(string), Some(app));
            let id = app.as_index();
            assert_eq!(SupportedApps::from_id(id), Some(app));
        }
    }

    #[test]
    fn test_cycling() {
        // Test forward cycling
        assert_eq!(SupportedApps::NixOS.next(), SupportedApps::BitcoinCore);
        assert_eq!(SupportedApps::WebUI.next(), SupportedApps::NixOS);

        // Test backward cycling
        assert_eq!(SupportedApps::BitcoinCore.previous(), SupportedApps::NixOS);
        assert_eq!(SupportedApps::NixOS.previous(), SupportedApps::WebUI);

        // Test a full cycle brings you back to the start
        let app = SupportedApps::CoreLightning;
        assert_eq!(app.next().previous(), app);
        assert_eq!(app.previous().next(), app);

        // Test a full 6-step cycle
        assert_eq!(app.next().next().next().next().next().next(), app);
    }
}
