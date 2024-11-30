use serde::{Deserialize, Serialize};

use super::option_data::OptionId;

#[derive(Clone, Default, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoolOptionData {
    pub id: OptionId,
    pub title: String,
    pub value: bool,
    pub dirty: bool,
    pub original: bool,
}

impl BoolOptionData {
    pub fn new(id: OptionId, title: String, value: bool, dirty: bool, original: bool) -> Self {
        Self {
            id,
            title,
            value,
            dirty,
            original,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoolOptionChangeData {
    pub id: OptionId,
    pub value: bool,
}

impl BoolOptionChangeData {
    pub fn new(id: OptionId, value: bool) -> Self {
        Self { id, value }
    }
}
