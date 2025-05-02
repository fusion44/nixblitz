use serde::{Deserialize, Serialize};

use super::option_data::{ApplicableOptionData, GetOptionId, OptionId, ToNixString};

#[derive(Clone, Default, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoolOptionData {
    /// Unique identifier for the option
    id: OptionId,

    /// Whether the option is currently applied to the system configuration
    applied: bool,

    /// Current value of the number option
    value: bool,

    /// Original value of the number option as applied to the system
    original: bool,
}

impl BoolOptionData {
    pub fn new(id: OptionId, value: bool) -> Self {
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

    pub fn value(&self) -> bool {
        self.value
    }

    pub fn set_value(&mut self, value: bool) {
        self.value = value;
        self.applied = value != self.original;
    }
}

impl ApplicableOptionData for BoolOptionData {
    fn set_applied(&mut self) {
        self.applied = false
    }
}

impl ToNixString for BoolOptionData {
    fn to_nix_string(&self, quote: bool) -> String {
        if quote {
            format!("\"{}\"", self.value)
        } else {
            self.value.to_string()
        }
    }
}

impl GetOptionId for BoolOptionData {
    fn id(&self) -> &OptionId {
        &self.id
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoolOptionChangeData {
    pub id: OptionId,
    pub value: bool,
}

impl BoolOptionChangeData {
    pub fn new(id: OptionId, value: bool) -> Self {
        Self { id, value }
    }
}

impl GetOptionId for BoolOptionChangeData {
    fn id(&self) -> &OptionId {
        &self.id
    }
}

#[cfg(test)]
mod tests {
    use crate::{app_option_data::option_data::ToOptionId, nix_base_config::NixBaseConfigOption};

    use super::*;

    #[test]
    fn test_bool_option_data_new() {
        let id = NixBaseConfigOption::AllowUnfree.to_option_id();
        let value = true;
        let bool_option = BoolOptionData::new(id.clone(), value);
        assert_eq!(bool_option.id(), &id);
        assert_eq!(bool_option.value(), value);
        assert!(!bool_option.is_applied());
    }

    #[test]
    fn test_bool_option_data_set_value() {
        let id = NixBaseConfigOption::AllowUnfree.to_option_id();
        let mut bool_option = BoolOptionData::new(id, false);
        bool_option.set_value(true);
        assert!(bool_option.value());
        assert!(bool_option.is_applied());
    }

    #[test]
    fn test_bool_option_data_to_nix_string() {
        let id = NixBaseConfigOption::AllowUnfree.to_option_id();
        let bool_option = BoolOptionData::new(id, true);
        assert_eq!(bool_option.to_nix_string(true), "\"true\"");
        assert_eq!(bool_option.to_nix_string(false), "true");
    }

    #[test]
    fn test_bool_option_change_data_new() {
        let id = NixBaseConfigOption::AllowUnfree.to_option_id();
        let value = true;
        let change_data = BoolOptionChangeData::new(id.clone(), value);
        assert_eq!(change_data.id(), &id);
        assert_eq!(change_data.value, value);
    }
}
