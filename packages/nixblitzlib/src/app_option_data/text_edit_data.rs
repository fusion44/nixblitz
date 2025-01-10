use serde::{Deserialize, Serialize};

use super::option_data::{GetOptionId, OptionId};

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Default, Debug)]
pub struct TextOptionData {
    /// Unique identifier for the text option
    id: OptionId,

    /// Current value of the text option
    value: String,

    /// Maximum number of lines allowed for the text option
    max_lines: u16,

    /// Indicates if the current value has been modified from the original
    /// since last rebuild from the system
    dirty: bool,

    /// Original value of the text option as applied to the system
    original: String,
}

impl TextOptionData {
    pub fn new(id: OptionId, value: String, max_lines: u16, dirty: bool, original: String) -> Self {
        let max_lines = if max_lines == 0 { 1 } else { max_lines };
        Self {
            id,
            value,
            max_lines,
            dirty,
            original,
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

    pub fn max_lines(&self) -> u16 {
        self.max_lines
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
