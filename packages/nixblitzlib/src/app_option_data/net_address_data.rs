use std::net::IpAddr;

use serde::{Deserialize, Serialize};

use super::option_data::{ApplicableOptionData, GetOptionId, OptionId, ToNixString};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetAddressOptionData {
    /// Unique identifier for the option
    id: OptionId,

    /// Whether the option is currently applied to the system configuration
    applied: bool,

    /// Current value of the number option
    value: Option<IpAddr>,

    /// Original value of the number option as applied to the system
    original: Option<IpAddr>,
}

impl ToNixString for NetAddressOptionData {
    fn to_nix_string(&self, quote: bool) -> String {
        match self.value {
            Some(value) => {
                if quote {
                    format!("\"{}\"", value).to_string()
                } else {
                    value.to_string()
                }
            }
            None => "null".to_string(),
        }
    }
}

impl NetAddressOptionData {
    pub fn new(id: OptionId, value: Option<IpAddr>) -> Self {
        Self {
            id,
            value,
            applied: false,
            original: value,
        }
    }

    pub fn is_applied(&self) -> bool {
        self.applied
    }

    pub fn value(&self) -> Option<IpAddr> {
        self.value
    }

    pub fn set_value(&mut self, value: Option<IpAddr>) {
        if self.value != value {
            self.value = value;
            self.applied = true;
        }
    }
}

impl ApplicableOptionData for NetAddressOptionData {
    fn set_applied(&mut self) {
        self.applied = false
    }
}

impl GetOptionId for NetAddressOptionData {
    fn id(&self) -> &OptionId {
        &self.id
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetAddressOptionChangeData {
    pub id: OptionId,
    pub value: Option<IpAddr>,
}

impl NetAddressOptionChangeData {
    pub fn new(id: OptionId, value: Option<IpAddr>) -> Self {
        Self { id, value }
    }
}

impl GetOptionId for NetAddressOptionChangeData {
    fn id(&self) -> &OptionId {
        &self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::IpAddr;
    use std::str::FromStr;

    #[test]
    fn test_net_address_option_data_new() {
        let id = OptionId {
            app: crate::apps::SupportedApps::BitcoinCore,
            option: "1".into(),
        };
        let ip = IpAddr::from_str("192.168.1.1").unwrap();
        let data = NetAddressOptionData::new(id.clone(), Some(ip));

        assert_eq!(data.id(), &id);
        assert_eq!(data.value(), Some(ip));
        assert!(!data.is_applied());
    }

    #[test]
    fn test_net_address_option_data_set_value() {
        let id = OptionId {
            app: crate::apps::SupportedApps::BitcoinCore,
            option: "1".into(),
        };
        let ip1 = IpAddr::from_str("192.168.1.1").unwrap();
        let ip2 = IpAddr::from_str("192.168.1.2").unwrap();
        let mut data = NetAddressOptionData::new(id, Some(ip1));

        data.set_value(Some(ip2));
        assert_eq!(data.value(), Some(ip2));
        assert!(data.is_applied());

        data.set_value(Some(ip2));
        assert_eq!(data.value(), Some(ip2));
        assert!(data.is_applied());
    }

    #[test]
    fn test_net_address_option_data_to_nix_string() {
        let id = OptionId {
            app: crate::apps::SupportedApps::BitcoinCore,
            option: "1".into(),
        };
        let ip = IpAddr::from_str("192.168.1.1").unwrap();
        let data = NetAddressOptionData::new(id.clone(), Some(ip));

        assert_eq!(data.to_nix_string(false), "192.168.1.1");
        assert_eq!(data.to_nix_string(true), "\"192.168.1.1\"");

        let data_null = NetAddressOptionData::new(id, None);
        assert_eq!(data_null.to_nix_string(false), "null");
    }

    #[test]
    fn test_net_address_option_change_data_new() {
        let id = OptionId {
            app: crate::apps::SupportedApps::BitcoinCore,
            option: "1".into(),
        };
        let ip = IpAddr::from_str("192.168.1.1").unwrap();
        let change_data = NetAddressOptionChangeData::new(id.clone(), Some(ip));

        assert_eq!(change_data.id(), &id);
        assert_eq!(change_data.value, Some(ip));
    }
}
