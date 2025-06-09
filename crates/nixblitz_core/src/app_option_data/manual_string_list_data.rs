use serde::{Deserialize, Serialize};

use super::option_data::{ApplicableOptionData, GetOptionId, OptionId, ToNixString};

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManualStringListOptionData {
    /// Unique identifier for the option
    id: OptionId,

    /// The current value of the string list
    value: Vec<String>,

    /// The original, unaltered value of the option as applied to the system
    original: Vec<String>,

    /// The maximum number of lines allowed in the string list
    max_lines: u16,

    /// Whether the option is currently applied to the system configuration
    applied: bool,
}

impl ManualStringListOptionData {
    /// Creates a new ManualStringListOptionData with the given
    /// id, values
    pub fn new(id: OptionId, value: Vec<String>, max_lines: u16) -> Self {
        Self {
            id,
            value: value.clone(),
            max_lines,
            applied: true,
            original: value,
        }
    }

    pub fn is_applied(&self) -> bool {
        self.applied
    }

    pub fn value(&self) -> &Vec<String> {
        &self.value
    }

    pub fn set_value(&mut self, value: Vec<String>) {
        self.applied = value == self.original;
        self.value = value;
    }

    pub fn max_lines(&self) -> u16 {
        self.max_lines
    }
}

impl ApplicableOptionData for ManualStringListOptionData {
    fn set_applied(&mut self) {
        self.original = self.value.clone();
        self.applied = true
    }
}

impl ToNixString for ManualStringListOptionData {
    fn to_nix_string(&self, quote: bool) -> String {
        if quote {
            format!("\"{}\"", self.value.join("\n"))
        } else {
            self.value.join("\n").clone()
        }
    }
}

impl GetOptionId for ManualStringListOptionData {
    fn id(&self) -> &OptionId {
        &self.id
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManualStringListOptionChangeData {
    pub id: OptionId,
    pub value: Vec<String>,
}

impl ManualStringListOptionChangeData {
    pub fn new(id: OptionId, value: Vec<String>) -> Self {
        Self { id, value }
    }
}

impl GetOptionId for ManualStringListOptionChangeData {
    fn id(&self) -> &OptionId {
        &self.id
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        app_option_data::option_data::ToOptionId, option_definitions::nix_base::NixBaseConfigOption,
    };

    use super::*;

    #[test]
    fn test_manual_string_list_option_data_new() {
        let id = NixBaseConfigOption::SshAuthKeys.to_option_id();
        let value = vec!["item1".to_string(), "item2".to_string()];
        let data = ManualStringListOptionData::new(id.clone(), value.clone(), 100);

        assert_eq!(data.id, id, "ID mismatch after new");
        assert_eq!(data.value, value, "Value mismatch after new");
        assert_eq!(data.original, value, "Original mismatch after new");
        assert!(data.applied, "Applied should be true after new");
    }

    #[test]
    fn test_manual_string_list_option_data_is_applied() {
        let id = NixBaseConfigOption::SshAuthKeys.to_option_id();
        let value = vec!["item1".to_string()];
        let data = ManualStringListOptionData::new(id, value, 100);

        assert!(
            data.is_applied(),
            "is_applied should be true immediately after creation"
        );
    }

    #[test]
    fn test_manual_string_list_option_data_value() {
        let id = NixBaseConfigOption::SshAuthKeys.to_option_id();
        let value = vec!["apple".to_string(), "banana".to_string()];
        let data = ManualStringListOptionData::new(id, value.clone(), 100);

        assert_eq!(data.value(), &value, "Value getter mismatch");
    }

    #[test]
    fn test_manual_string_list_option_data_set_value_different_from_original() {
        let id = NixBaseConfigOption::SshAuthKeys.to_option_id();
        let original_value = vec!["old_item".to_string()];
        let mut data = ManualStringListOptionData::new(id, original_value.clone(), 100);

        assert!(data.is_applied(), "Should be applied right after new");

        let new_value = vec!["new_item".to_string(), "another_item".to_string()];
        data.set_value(new_value.clone());

        assert_eq!(
            data.value(),
            &new_value,
            "Value mismatch after setting different value"
        );
        assert!(
            !data.is_applied(),
            "Applied should be false when value is different from original"
        );
    }

    #[test]
    fn test_manual_string_list_option_data_set_value_same_as_original() {
        let id = NixBaseConfigOption::SshAuthKeys.to_option_id();
        let original_value = vec!["same_item".to_string()];
        let mut data = ManualStringListOptionData::new(id, original_value.clone(), 100);

        // At this point, `data.applied` should be true because `new` sets it.
        assert!(data.is_applied(), "Should be applied right after new");

        data.set_value(original_value.clone());

        assert_eq!(
            data.value(),
            &original_value,
            "Value mismatch after setting same value"
        );
        // `applied` should remain true because `value == original`
        assert!(
            data.is_applied(),
            "Applied should be true when value is same as original"
        );
    }

    #[test]
    fn test_manual_string_list_option_data_set_applied() {
        let id = NixBaseConfigOption::SshAuthKeys.to_option_id();
        let initial_value = vec!["initial".to_string()];
        let mut data = ManualStringListOptionData::new(id, initial_value.clone(), 100);

        // Change the value to make it "unapplied" (according to the new logic of `set_value`)
        let changed_value = vec!["changed".to_string()];
        data.set_value(changed_value.clone());
        assert!(
            !data.is_applied(),
            "Should be unapplied after changing value"
        );
        assert_eq!(
            data.original, initial_value,
            "Original should remain initial value"
        );
        assert_eq!(data.value, changed_value, "Value should be changed value");

        // Call set_applied
        data.set_applied();
        assert!(data.is_applied(), "Should be applied after set_applied");
        // original should now be equal to value
        assert_eq!(
            data.original, changed_value,
            "Original should be updated to current value after set_applied"
        );
        assert_eq!(
            data.value, changed_value,
            "Value should remain changed value"
        );
    }

    #[test]
    fn test_manual_string_list_option_data_to_nix_string_quoted() {
        let id = NixBaseConfigOption::SshAuthKeys.to_option_id();
        let value = vec![
            "line1".to_string(),
            "line2".to_string(),
            "line3".to_string(),
        ];
        let data = ManualStringListOptionData::new(id, value, 100);

        let nix_string = data.to_nix_string(true);
        assert_eq!(
            nix_string, "\"line1\nline2\nline3\"",
            "Quoted Nix string mismatch"
        );
    }

    #[test]
    fn test_manual_string_list_option_data_to_nix_string_unquoted() {
        let id = NixBaseConfigOption::SshAuthKeys.to_option_id();
        let value = vec!["single_line".to_string()];
        let data = ManualStringListOptionData::new(id, value, 100);

        let nix_string = data.to_nix_string(false);
        assert_eq!(nix_string, "single_line", "Unquoted Nix string mismatch");
    }

    #[test]
    fn test_manual_string_list_option_data_get_option_id() {
        let id = NixBaseConfigOption::SshAuthKeys.to_option_id();
        let value = vec!["test".to_string()];
        let data = ManualStringListOptionData::new(id.clone(), value, 100);

        assert_eq!(data.id(), &id, "Option ID getter mismatch");
    }

    #[test]
    fn test_manual_string_list_option_change_data_new() {
        let id = NixBaseConfigOption::SshAuthKeys.to_option_id();
        let value = vec!["change_item1".to_string(), "change_item2".to_string()];
        let change_data = ManualStringListOptionChangeData::new(id.clone(), value.clone());

        assert_eq!(change_data.id, id, "Change data ID mismatch");
        assert_eq!(change_data.value, value, "Change data value mismatch");
    }

    #[test]
    fn test_manual_string_list_option_change_data_get_option_id() {
        let id = NixBaseConfigOption::SshAuthKeys.to_option_id();
        let value = vec!["another_change".to_string()];
        let change_data = ManualStringListOptionChangeData::new(id.clone(), value);

        assert_eq!(
            change_data.id(),
            &id,
            "Change data Option ID getter mismatch"
        );
    }
}
