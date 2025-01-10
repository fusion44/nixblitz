use serde::{Deserialize, Serialize};

use crate::number_value::NumberValue;

use super::option_data::{GetOptionId, OptionId, ToNixString};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PortOptionData {
    id: OptionId,
    dirty: bool,
    value: NumberValue,
    original: NumberValue,
}

impl PortOptionData {
    pub fn new(id: OptionId, value: NumberValue) -> Self {
        Self {
            id,
            value: value.clone(),
            dirty: false,
            original: value,
        }
    }

    pub fn dirty(&self) -> bool {
        self.dirty
    }

    pub fn value(&self) -> &NumberValue {
        &self.value
    }

    pub fn set_value(&mut self, value: NumberValue) {
        if self.value != value {
            self.value = value;
            self.dirty = true;
        }
    }
}

impl GetOptionId for PortOptionData {
    fn id(&self) -> &OptionId {
        &self.id
    }
}

impl ToNixString for PortOptionData {
    fn to_nix_string(&self, quote: bool) -> String {
        let value = match self.value {
            NumberValue::U16(v) => v.map_or_else(|| "null".to_string(), |val| val.to_string()),
            NumberValue::UInt(v) => v.map_or_else(|| "null".to_string(), |val| val.to_string()),
            NumberValue::Int(v) => v.map_or_else(|| "null".to_string(), |val| val.to_string()),
            NumberValue::Float(v) => v.map_or_else(|| "null".to_string(), |val| val.to_string()),
        };

        if quote {
            format!("\"{}\"", value)
        } else {
            value
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PortOptionChangeData {
    pub id: OptionId,
    pub value: NumberValue,
}

impl PortOptionChangeData {
    pub fn new(id: OptionId, value: NumberValue) -> Self {
        Self { id, value }
    }
}

impl GetOptionId for PortOptionChangeData {
    fn id(&self) -> &OptionId {
        &self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::number_value::NumberValue;

    #[test]
    fn test_port_option_data_new() {
        let id = OptionId::new(crate::apps::SupportedApps::BitcoinCore, "test".into());
        let value = NumberValue::UInt(Some(42));
        let port_option_data = PortOptionData::new(id.clone(), value.clone());

        assert_eq!(port_option_data.id(), &id);
        assert_eq!(port_option_data.value(), &value);
        assert!(!port_option_data.dirty());
    }

    #[test]
    fn test_port_option_data_set_value() {
        let id = OptionId::new(crate::apps::SupportedApps::BitcoinCore, "test".into());
        let mut port_option_data = PortOptionData::new(id, NumberValue::UInt(Some(42)));

        port_option_data.set_value(NumberValue::UInt(Some(43)));
        assert_eq!(port_option_data.value(), &NumberValue::UInt(Some(43)));
        assert!(port_option_data.dirty());

        port_option_data.set_value(NumberValue::UInt(Some(43)));
        assert_eq!(port_option_data.value(), &NumberValue::UInt(Some(43)));
        assert!(port_option_data.dirty());
    }

    #[test]
    fn test_port_option_data_to_nix_string() {
        let id = OptionId::new(crate::apps::SupportedApps::BitcoinCore, "test".into());
        let port_option_data = PortOptionData::new(id, NumberValue::UInt(Some(42)));

        assert_eq!(port_option_data.to_nix_string(false), "42");
        assert_eq!(port_option_data.to_nix_string(true), "\"42\"");
    }

    #[test]
    fn test_port_option_change_data_new() {
        let id = OptionId::new(crate::apps::SupportedApps::BitcoinCore, "test".into());
        let value = NumberValue::UInt(Some(42));
        let port_option_change_data = PortOptionChangeData::new(id.clone(), value.clone());

        assert_eq!(port_option_change_data.id(), &id);
        assert_eq!(port_option_change_data.value, value);
    }
}
