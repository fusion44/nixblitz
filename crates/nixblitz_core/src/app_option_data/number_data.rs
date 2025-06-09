use serde::{Deserialize, Serialize};

use crate::{errors::ArgumentError, number_value::NumberValue};

use super::option_data::{ApplicableOptionData, GetOptionId, OptionId, ToNixString};

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct NumberOptionData {
    /// Unique identifier for the option
    id: OptionId,

    /// Current value of the number option
    value: NumberValue,

    /// Least possible value
    range_min: usize,

    /// Max possible value
    range_max: usize,

    /// Whether the option is currently applied to the system configuration
    applied: bool,

    /// Original value of the number option as applied to the system
    original: NumberValue,
}

impl NumberOptionData {
    pub fn new(
        id: OptionId,
        value: NumberValue,
        range_min: usize,
        range_max: usize,
        applied: bool,
        original: NumberValue,
    ) -> Result<Self, ArgumentError> {
        Ok(Self {
            id,
            value,
            range_min,
            range_max,
            applied,
            original,
        })
    }

    pub fn is_applied(&self) -> bool {
        self.applied
    }

    pub fn value(&self) -> &NumberValue {
        &self.value
    }

    pub fn set_value(&mut self, value: NumberValue) {
        if value != self.value {
            self.applied = value != self.original;
            self.value = value;
        }
    }

    pub fn range_min(&self) -> usize {
        self.range_min
    }

    pub fn range_max(&self) -> usize {
        self.range_max
    }
}

impl ApplicableOptionData for NumberOptionData {
    fn set_applied(&mut self) {
        self.applied = false
    }
}

impl ToNixString for NumberOptionData {
    fn to_nix_string(&self, quote: bool) -> String {
        if quote {
            format!("\"{}\"", self.value)
        } else {
            self.value.to_string()
        }
    }
}

impl GetOptionId for NumberOptionData {
    fn id(&self) -> &OptionId {
        &self.id
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NumberOptionChangeData {
    pub id: OptionId,
    pub value: NumberValue,
}

impl NumberOptionChangeData {
    pub fn new(id: OptionId, value: NumberValue) -> Self {
        Self { id, value }
    }
}

impl GetOptionId for NumberOptionChangeData {
    fn id(&self) -> &OptionId {
        &self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{apps::SupportedApps, number_value::NumberValue};

    #[test]
    fn test_number_option_data_new() {
        let id = OptionId::new(SupportedApps::BitcoinCore, "test".into());
        let value = NumberValue::UInt(Some(10));
        let original = NumberValue::UInt(Some(10));
        let number_option =
            NumberOptionData::new(id.clone(), value.clone(), 0, 100, false, original.clone())
                .unwrap();

        assert!(!number_option.is_applied());
        assert_eq!(number_option.id(), &id);
        assert_eq!(number_option.value(), &value);
        assert_eq!(number_option.range_min(), 0);
        assert_eq!(number_option.range_max(), 100);
    }

    #[test]
    fn test_number_option_data_set_value() {
        let id = OptionId::new(SupportedApps::BitcoinCore, "test".into());
        let original = NumberValue::UInt(Some(10));
        let mut number_option =
            NumberOptionData::new(id, original.clone(), 0, 100, false, original.clone()).unwrap();

        let new_value = NumberValue::UInt(Some(20));
        number_option.set_value(new_value.clone());
        assert_eq!(number_option.value(), &new_value);
        assert!(number_option.is_applied());

        // Setting the value back to original should reset applied flag
        number_option.set_value(original.clone());
        assert_eq!(number_option.value(), &original);
        assert!(!number_option.is_applied());
    }

    #[test]
    fn test_number_option_data_range() {
        let id = OptionId::new(SupportedApps::BitcoinCore, "test".into());
        let value = NumberValue::UInt(Some(10));
        let original = NumberValue::UInt(Some(10));
        let number_option = NumberOptionData::new(id, value, 5, 50, false, original).unwrap();

        assert_eq!(number_option.range_min(), 5);
        assert_eq!(number_option.range_max(), 50);
    }

    #[test]
    fn test_to_nix_string() {
        let id = OptionId::new(SupportedApps::BitcoinCore, "test".into());
        let value = NumberValue::UInt(Some(10));
        let original = NumberValue::UInt(Some(10));
        let number_option = NumberOptionData::new(id, value, 0, 100, false, original).unwrap();

        let quoted_string = number_option.to_nix_string(true);
        assert_eq!(quoted_string, "\"10\"");

        let unquoted_string = number_option.to_nix_string(false);
        assert_eq!(unquoted_string, "10");
    }
}
