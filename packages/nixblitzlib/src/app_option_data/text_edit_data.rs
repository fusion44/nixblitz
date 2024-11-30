use serde::{Deserialize, Serialize};

use super::option_data::OptionId;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Default, Debug)]
pub struct TextOptionData {
    pub id: OptionId,
    pub title: String,
    pub value: String,
    pub max_lines: u16,
    pub dirty: bool,
    pub original: String,
}

impl TextOptionData {
    pub fn new(
        id: OptionId,
        title: String,
        value: String,
        max_lines: u16,
        dirty: bool,
        original: String,
    ) -> Self {
        Self {
            id,
            title,
            value,
            max_lines,
            dirty,
            original,
        }
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
