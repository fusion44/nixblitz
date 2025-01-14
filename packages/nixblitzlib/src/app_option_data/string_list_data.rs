use serde::{Deserialize, Serialize};

use super::option_data::{GetOptionId, OptionId, ToNixString};

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StringListOptionItem {
    /// The value this item represents
    pub value: String,

    /// The name that is displayed to the user
    pub display_name: String,
}

impl StringListOptionItem {
    /// Creates a new StringListOptionItem with the given value and display name
    pub fn new(value: String, display_name: String) -> Self {
        Self {
            value,
            display_name,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StringListOptionData {
    /// The id of the option
    id: OptionId,

    /// The current value of the option
    value: String,

    /// The original, unaltered value of the option
    original: String,

    /// The allowed values the option can take
    options: Vec<StringListOptionItem>,

    /// Whether the option is currently dirty (not yet saved)
    dirty: bool,
}

impl StringListOptionData {
    /// Creates a new StringListOptionData with the given
    /// id, title, value, and options
    pub fn new(id: OptionId, value: String, options: Vec<StringListOptionItem>) -> Self {
        Self {
            id,
            value: value.clone(),
            options,
            dirty: false,
            original: value,
        }
    }

    pub fn dirty(&self) -> bool {
        self.dirty
    }

    pub fn value(&self) -> &str {
        self.value.as_str()
    }

    pub fn set_value(&mut self, value: String) {
        self.dirty = value != self.original;
        self.value = value;
    }

    pub fn options(&self) -> &Vec<StringListOptionItem> {
        &self.options
    }
}

impl ToNixString for StringListOptionData {
    fn to_nix_string(&self, quote: bool) -> String {
        if quote {
            format!("\"{}\"", self.value)
        } else {
            self.value.clone()
        }
    }
}

impl GetOptionId for StringListOptionData {
    fn id(&self) -> &OptionId {
        &self.id
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StringListOptionChangeData {
    pub id: OptionId,
    pub value: String,
}

impl StringListOptionChangeData {
    pub fn new(id: OptionId, value: String) -> Self {
        Self { id, value }
    }
}

impl GetOptionId for StringListOptionChangeData {
    fn id(&self) -> &OptionId {
        &self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_list_option_item_new() {
        let item = StringListOptionItem::new("value".to_string(), "display".to_string());
        assert_eq!(item.value, "value");
        assert_eq!(item.display_name, "display");
    }

    #[test]
    fn test_string_list_option_data_new() {
        let id = OptionId::default();
        let options = vec![
            StringListOptionItem::new("value1".to_string(), "display1".to_string()),
            StringListOptionItem::new("value2".to_string(), "display2".to_string()),
        ];
        let data = StringListOptionData::new(id.clone(), "value1".to_string(), options.clone());
        assert_eq!(data.id, id);
        assert_eq!(data.value, "value1");
        assert_eq!(data.original, "value1");
        assert_eq!(data.options, options);
        assert!(!data.dirty);
    }

    #[test]
    fn test_string_list_option_data_set_value() {
        let id = OptionId::default();
        let options = vec![
            StringListOptionItem::new("value1".to_string(), "display1".to_string()),
            StringListOptionItem::new("value2".to_string(), "display2".to_string()),
        ];
        let mut data = StringListOptionData::new(id, "value1".to_string(), options);
        data.set_value("value2".to_string());
        assert_eq!(data.value(), "value2");
        assert!(data.dirty());
    }

    #[test]
    fn test_string_list_option_data_to_nix_string() {
        let id = OptionId::default();
        let options = vec![
            StringListOptionItem::new("value1".to_string(), "display1".to_string()),
            StringListOptionItem::new("value2".to_string(), "display2".to_string()),
        ];
        let data = StringListOptionData::new(id, "value1".to_string(), options);
        assert_eq!(data.to_nix_string(true), "\"value1\"");
        assert_eq!(data.to_nix_string(false), "value1");
    }

    #[test]
    fn test_string_list_option_change_data_new() {
        let id = OptionId::default();
        let change_data = StringListOptionChangeData::new(id.clone(), "new_value".to_string());
        assert_eq!(change_data.id, id);
        assert_eq!(change_data.value, "new_value");
    }
}
