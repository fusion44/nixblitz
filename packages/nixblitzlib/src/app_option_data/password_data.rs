use serde::{Deserialize, Serialize};

use super::option_data::{GetOptionId, OptionId, ToNixString};

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Default, Debug)]
pub struct PasswordOptionData {
    /// Unique identifier for the text option
    id: OptionId,

    /// Current hashed value of the password
    hashed_value: String,

    /// Whether to ask the user to confirm the password (e.g. new passwords)
    confirm: bool,

    /// The min length the password must have
    min_length: usize,

    /// Indicates if the current value has been modified from the original
    /// since last rebuild from the system
    dirty: bool,

    /// Am optional to display in the option menu
    subtitle: String,
}

impl PasswordOptionData {
    pub fn new(
        id: OptionId,
        hashed_value: String,
        confirm: bool,
        min_length: usize,
        dirty: bool,
        subtitle: String,
    ) -> Self {
        Self {
            id,
            hashed_value,
            confirm,
            min_length,
            dirty,
            subtitle,
        }
    }

    pub fn dirty(&self) -> bool {
        self.dirty
    }

    pub fn confirm(&self) -> bool {
        self.confirm
    }

    pub fn min_length(&self) -> usize {
        self.min_length
    }

    pub fn subtitle(&self) -> String {
        self.subtitle.clone()
    }

    pub fn set_subtitle(&mut self, value: String) {
        self.subtitle = value;
    }

    pub fn hashed_value(&self) -> &String {
        &self.hashed_value
    }

    pub fn set_hashed_value(&mut self, value: String) {
        self.hashed_value = value;
    }
}

impl ToNixString for PasswordOptionData {
    fn to_nix_string(&self, quote: bool) -> String {
        if quote {
            format!("\"{}\"", self.hashed_value)
        } else {
            self.hashed_value().to_string()
        }
    }
}

impl GetOptionId for PasswordOptionData {
    fn id(&self) -> &OptionId {
        &self.id
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PasswordOptionChangeData {
    pub id: OptionId,
    pub value: String,
    pub confirm: Option<String>,
}

impl PasswordOptionChangeData {
    pub fn new(id: OptionId, value: String, confirm: Option<String>) -> Self {
        Self { id, value, confirm }
    }
}

impl GetOptionId for PasswordOptionChangeData {
    fn id(&self) -> &OptionId {
        &self.id
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        app_option_data::{
            option_data::{ToNixString, ToOptionId},
            password_data::PasswordOptionData,
        },
        nix_base_config::NixBaseConfigOption,
        utils::unix_hash_password,
    };

    fn test_password_option_data_creation() {
        let id = NixBaseConfigOption::InitialPassword.to_option_id();
        let hashed_value = "hashed_password".to_string();
        let confirm = true;
        let min_length = 8;
        let dirty = false;
        let subtitle = "Test Subtitle".to_string();

        let password_option = PasswordOptionData::new(
            id.clone(),
            hashed_value.clone(),
            confirm,
            min_length,
            dirty,
            subtitle.clone(),
        );

        assert_eq!(password_option.id, id);
        assert_eq!(password_option.hashed_value(), &hashed_value);
        assert_eq!(password_option.confirm(), confirm);
        assert_eq!(password_option.min_length(), min_length);
        assert_eq!(password_option.dirty(), dirty);
        assert_eq!(password_option.subtitle(), subtitle);
    }

    #[test]
    fn test_password_option_data_setters() {
        let id = NixBaseConfigOption::InitialPassword.to_option_id();
        let pw = unix_hash_password("initial password").unwrap();
        let mut password_option =
            PasswordOptionData::new(id, pw.clone(), false, 8, false, pw.clone());

        // user is responsible to hash the value, so we can set just "new_hash"
        password_option.set_hashed_value("new_hash".to_string());
        assert_eq!(password_option.hashed_value(), "new_hash");

        password_option.set_subtitle("New Subtitle".to_string());
        assert_eq!(password_option.subtitle(), "New Subtitle");
    }

    #[test]
    fn test_password_option_data_to_nix_string() {
        let id = NixBaseConfigOption::InitialPassword.to_option_id();
        let pw = unix_hash_password("initial password").unwrap();
        let password_option =
            PasswordOptionData::new(id, pw.clone(), false, 8, false, "".to_string());

        assert_eq!(password_option.to_nix_string(true), format!("\"{}\"", pw));
        assert_eq!(password_option.to_nix_string(false), format!("{}", pw));
    }
}
