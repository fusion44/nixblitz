use serde::{Deserialize, Serialize};

use super::option_data::{GetOptionId, OptionId};

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
