use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub enum SupportedApps {
    BitcoinCore,
    CoreLightning,
    LND,
    BlitzAPI,
    WebUI,
}

impl SupportedApps {
    const APP_NAMES: [&'static str; 5] = [
        "Bitcoin Core",
        "Core Lightning",
        "LND",
        "Blitz Api",
        "Web UI",
    ];

    pub fn from(s: &str) -> Option<Self> {
        if s == Self::APP_NAMES[0] {
            return Some(SupportedApps::BitcoinCore);
        } else if s == Self::APP_NAMES[1] {
            return Some(SupportedApps::CoreLightning);
        } else if s == Self::APP_NAMES[2] {
            return Some(SupportedApps::LND);
        } else if s == Self::APP_NAMES[3] {
            return Some(SupportedApps::BlitzAPI);
        } else if s == Self::APP_NAMES[4] {
            return Some(SupportedApps::WebUI);
        }

        None
    }

    pub fn from_id(id: usize) -> Option<Self> {
        if id == 0 {
            return Some(SupportedApps::BitcoinCore);
        } else if id == 1 {
            return Some(SupportedApps::CoreLightning);
        } else if id == 2 {
            return Some(SupportedApps::LND);
        } else if id == 3 {
            return Some(SupportedApps::BlitzAPI);
        } else if id == 4 {
            return Some(SupportedApps::WebUI);
        }

        None
    }

    pub fn to_string(&self) -> &'static str {
        match self {
            SupportedApps::BitcoinCore => Self::APP_NAMES[0],
            SupportedApps::CoreLightning => Self::APP_NAMES[1],
            SupportedApps::LND => Self::APP_NAMES[2],
            SupportedApps::BlitzAPI => Self::APP_NAMES[3],
            SupportedApps::WebUI => Self::APP_NAMES[4],
        }
    }

    pub fn as_string_list() -> &'static [&'static str] {
        &Self::APP_NAMES
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_round_trip() {
        for app in [
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
