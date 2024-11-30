use core::fmt;

use serde::{Deserialize, Serialize};

#[derive(Default, Debug, PartialEq, Eq, Serialize, Deserialize, Clone, Copy)]
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
    const APP_NAMES: [&'static str; 6] = [
        "Nix OS",
        "Bitcoin Core",
        "Core Lightning",
        "LND",
        "Blitz Api",
        "Web UI",
    ];

    pub fn from(s: &str) -> Option<Self> {
        if s == Self::APP_NAMES[0] {
            return Some(SupportedApps::NixOS);
        } else if s == Self::APP_NAMES[1] {
            return Some(SupportedApps::BitcoinCore);
        } else if s == Self::APP_NAMES[2] {
            return Some(SupportedApps::CoreLightning);
        } else if s == Self::APP_NAMES[3] {
            return Some(SupportedApps::LND);
        } else if s == Self::APP_NAMES[4] {
            return Some(SupportedApps::BlitzAPI);
        } else if s == Self::APP_NAMES[5] {
            return Some(SupportedApps::WebUI);
        }

        None
    }

    pub fn from_id(id: usize) -> Option<Self> {
        if id == 0 {
            return Some(SupportedApps::NixOS);
        } else if id == 1 {
            return Some(SupportedApps::BitcoinCore);
        } else if id == 2 {
            return Some(SupportedApps::CoreLightning);
        } else if id == 3 {
            return Some(SupportedApps::LND);
        } else if id == 4 {
            return Some(SupportedApps::BlitzAPI);
        } else if id == 5 {
            return Some(SupportedApps::WebUI);
        }

        None
    }

    pub fn to_string(&self) -> &'static str {
        match self {
            SupportedApps::NixOS => Self::APP_NAMES[0],
            SupportedApps::BitcoinCore => Self::APP_NAMES[1],
            SupportedApps::CoreLightning => Self::APP_NAMES[2],
            SupportedApps::LND => Self::APP_NAMES[3],
            SupportedApps::BlitzAPI => Self::APP_NAMES[4],
            SupportedApps::WebUI => Self::APP_NAMES[5],
        }
    }

    pub fn as_string_list() -> &'static [&'static str] {
        &Self::APP_NAMES
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
        for app in [
            SupportedApps::NixOS,
            SupportedApps::BitcoinCore,
            SupportedApps::CoreLightning,
            SupportedApps::LND,
            SupportedApps::BlitzAPI,
            SupportedApps::WebUI,
        ] {
            let string = app.to_string();
            assert_eq!(SupportedApps::from(string), Some(app));
        }
    }
}
