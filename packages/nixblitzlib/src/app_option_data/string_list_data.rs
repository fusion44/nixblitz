use serde::{Deserialize, Serialize};

use super::option_data::OptionId;

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
    pub id: OptionId,

    /// The title of the option
    pub title: String,

    /// The current value of the option
    pub value: String,

    /// The original, unaltered value of the option
    pub original: String,

    /// The allowed values the option can take
    pub options: Vec<StringListOptionItem>,

    /// Whether the option is currently dirty (not yet saved)
    pub dirty: bool,
}

impl StringListOptionData {
    /// Creates a new StringListOptionData with the given
    /// id, title, value, and options
    pub fn new(
        id: OptionId,
        title: String,
        value: String,
        original: String,
        options: Vec<StringListOptionItem>,
        dirty: bool,
    ) -> Self {
        Self {
            id,
            title,
            value,
            options,
            dirty,
            original,
        }
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
