use serde::{Deserialize, Serialize};

use super::option_data::{GetOptionId, OptionId};

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
