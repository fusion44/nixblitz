use serde::{Deserialize, Serialize};

use super::option_data::{ApplicableOptionData, GetOptionId, OptionId, ToNixString};

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Default, Debug)]
pub struct TextOptionData {
    /// Unique identifier for the option
    id: OptionId,

    /// Current value of the text option
    value: String,

    /// Maximum number of lines allowed for the text option
    max_lines: u16,

    /// Whether the option is currently applied to the system configuration
    applied: bool,

    /// Original value of the text option as applied to the system
    original: String,
}

impl TextOptionData {
    pub fn new(
        id: OptionId,
        value: String,
        max_lines: u16,
        applied: bool,
        original: String,
    ) -> Self {
        let max_lines = if max_lines == 0 { 1 } else { max_lines };
        Self {
            id,
            value,
            max_lines,
            applied,
            original,
        }
    }

    pub fn is_applied(&self) -> bool {
        self.applied
    }

    pub fn value(&self) -> &str {
        self.value.as_str()
    }

    pub fn set_value(&mut self, value: String) {
        self.applied = value != self.original;
        self.value = value;
    }

    pub fn max_lines(&self) -> u16 {
        self.max_lines
    }
}

impl ApplicableOptionData for TextOptionData {
    fn set_applied(&mut self) {
        self.applied = false
    }
}

impl ToNixString for TextOptionData {
    fn to_nix_string(&self, quote: bool) -> String {
        if quote {
            format!("\"{}\"", self.value)
        } else {
            self.value.clone()
        }
    }
}

impl GetOptionId for TextOptionData {
    fn id(&self) -> &OptionId {
        &self.id
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextOptionChangeData {
    pub id: OptionId,
    pub value: String,
}

impl TextOptionChangeData {
    pub fn new(id: OptionId, value: String) -> Self {
        Self { id, value }
    }
}

impl GetOptionId for TextOptionChangeData {
    fn id(&self) -> &OptionId {
        &self.id
    }
}

#[cfg(test)]
mod tests {
    use crate::{app_option_data::option_data::ToOptionId, apps::SupportedApps};

    use super::*;

    pub enum TestOption {
        Username,
    }

    impl ToOptionId for TestOption {
        fn to_option_id(&self) -> OptionId {
            OptionId::new(SupportedApps::NixOS, "username".into())
        }
    }

    #[test]
    fn test_text_option_data_new() {
        let id = TestOption::Username.to_option_id();
        let value = String::from("test");
        let max_lines = 5;
        let applied = false;
        let original = String::from("test");

        let text_option = TextOptionData::new(
            id.clone(),
            value.clone(),
            max_lines,
            applied,
            original.clone(),
        );

        assert_eq!(text_option.id(), &id);
        assert_eq!(text_option.value(), value);
        assert_eq!(text_option.max_lines(), max_lines);
        assert_eq!(text_option.is_applied(), applied);
        assert_eq!(text_option.original, original);
    }

    #[test]
    fn test_text_option_data_set_value() {
        let id = TestOption::Username.to_option_id();
        let original = String::from("original");
        let mut text_option = TextOptionData::new(id, original.clone(), 5, false, original.clone());

        text_option.set_value(String::from("new value"));
        assert!(text_option.is_applied());
        assert_eq!(text_option.value(), "new value");

        text_option.set_value(original.clone());
        assert!(!text_option.is_applied());
        assert_eq!(text_option.value(), original);
    }

    #[test]
    fn test_to_nix_string() {
        let id = TestOption::Username.to_option_id();
        let value = String::from("test");
        let text_option = TextOptionData::new(id, value.clone(), 5, false, value.clone());

        assert_eq!(text_option.to_nix_string(true), "\"test\"");
        assert_eq!(text_option.to_nix_string(false), "test");
    }

    #[test]
    fn test_text_option_change_data_new() {
        let id = TestOption::Username.to_option_id();
        let value = String::from("change");
        let change_data = TextOptionChangeData::new(id.clone(), value.clone());

        assert_eq!(change_data.id(), &id);
        assert_eq!(change_data.value, value);
    }
}
