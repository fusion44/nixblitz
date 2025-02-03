use serde::{Deserialize, Serialize};

use super::option_data::{GetOptionId, OptionId, ToNixString};

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Default, Debug)]
pub struct PathOptionData {
    /// Unique identifier for the path option
    id: OptionId,

    /// Current value of the path option
    value: Option<String>,

    /// The default value of the option
    default: Option<String>,

    /// Indicates if the current value has been modified from the original
    /// since last rebuild from the system
    dirty: bool,

    /// Original value of the path option as applied to the system
    original: Option<String>,
}

impl PathOptionData {
    pub fn new(
        id: OptionId,
        value: Option<String>,
        default: Option<String>,
        dirty: bool,
        original: Option<String>,
    ) -> Self {
        Self {
            id,
            value,
            default,
            dirty,
            original,
        }
    }

    pub fn default_from(id: OptionId, default: Option<String>) -> Self {
        Self::new(id, default.clone(), default.clone(), false, default)
    }

    pub fn default(&self) -> Option<String> {
        self.default.clone()
    }

    pub fn dirty(&self) -> bool {
        self.dirty
    }

    pub fn value(&self) -> Option<String> {
        self.value.clone()
    }

    pub fn set_value(&mut self, value: Option<String>) {
        self.dirty = value != self.original;
        self.value = value;
    }
}

impl ToNixString for PathOptionData {
    /// Converts the current value of the path option to a Nix-compatible value.
    ///
    /// # Arguments
    ///
    /// * `quote` - A boolean indicating whether the resulting string should be quoted.
    ///
    /// # Returns
    ///
    /// A `String` representing the current value of the path option. If `quote` is true,
    /// the value will be enclosed in double quotes. If the value is `None`, it returns
    /// an unquoted "null".
    fn to_nix_string(&self, quote: bool) -> String {
        if let Some(value) = &self.value {
            if quote {
                return format!("\"{}\"", value);
            } else {
                return value.clone();
            }
        }

        "null".to_string()
    }
}

impl GetOptionId for PathOptionData {
    fn id(&self) -> &OptionId {
        &self.id
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PathOptionChangeData {
    pub id: OptionId,
    pub value: Option<String>,
}

impl PathOptionChangeData {
    pub fn new(id: OptionId, value: Option<String>) -> Self {
        Self { id, value }
    }
}

impl GetOptionId for PathOptionChangeData {
    fn id(&self) -> &OptionId {
        &self.id
    }
}

#[cfg(test)]
mod tests {
    use crate::{app_option_data::option_data::ToOptionId, nix_base_config::NixBaseConfigOption};

    use super::*;

    #[test]
    fn test_path_option_data_new() {
        let id = NixBaseConfigOption::Username.to_option_id();
        let value = Some(String::from("/tmp/some/folder"));
        let dirty = false;
        let original = Some(String::from("/tmp/some/folder"));
        let default = Some(String::from("/tmp/default_folder"));

        let path_option = PathOptionData::new(
            id.clone(),
            value.clone(),
            default.clone(),
            dirty,
            original.clone(),
        );

        assert_eq!(path_option.id(), &id);
        assert_eq!(path_option.value(), value);
        assert_eq!(path_option.default(), default);
        assert_eq!(path_option.dirty(), dirty);
        assert_eq!(path_option.original, original);
    }

    #[test]
    fn test_path_option_data_set_value() {
        let id = NixBaseConfigOption::Username.to_option_id();
        let original = Some(String::from("original"));
        let mut path_option =
            PathOptionData::new(id, original.clone(), None, false, original.clone());

        path_option.set_value(Some(String::from("new value")));
        assert!(path_option.dirty());
        assert_eq!(path_option.value(), Some("new value".to_string()));

        path_option.set_value(original.clone());
        assert!(!path_option.dirty());
        assert_eq!(path_option.value(), original);
    }

    #[test]
    fn test_to_nix_string() {
        let id = NixBaseConfigOption::Username.to_option_id();
        let value = Some(String::from("test"));
        let path_option = PathOptionData::new(id, value.clone(), None, false, value.clone());

        assert_eq!(path_option.to_nix_string(true), "\"test\"");
        assert_eq!(path_option.to_nix_string(false), "test");
    }

    #[test]
    fn test_to_nix_string_with_none_value() {
        let id = NixBaseConfigOption::Username.to_option_id();
        let path_opt = PathOptionData::new(id, None, None, false, None);

        assert_eq!(path_opt.to_nix_string(true), "null");
        assert_eq!(path_opt.to_nix_string(false), "null");
    }

    #[test]
    fn test_path_option_change_data_new() {
        let id = NixBaseConfigOption::Username.to_option_id();
        let value = Some(String::from("change"));
        let change_data = PathOptionChangeData::new(id.clone(), value.clone());

        assert_eq!(change_data.id(), &id);
        assert_eq!(change_data.value, value);
    }
}
