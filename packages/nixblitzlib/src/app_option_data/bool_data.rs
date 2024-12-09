use serde::{Deserialize, Serialize};

use super::option_data::{GetOptionId, OptionId};

#[derive(Clone, Default, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoolOptionData {
    id: OptionId,
    dirty: bool,
    value: bool,
    original: bool,
}

impl BoolOptionData {
    pub fn new(id: OptionId, value: bool) -> Self {
        Self {
            id,
            value,
            dirty: false,
            original: value,
        }
    }

    pub fn dirty(&self) -> bool {
        self.dirty
    }

    pub fn value(&self) -> bool {
        self.value
    }

    pub fn set_value(&mut self, value: bool) {
        self.value = value;
        self.dirty = value != self.original;
    }
}

impl GetOptionId for BoolOptionData {
    fn id(&self) -> &OptionId {
        &self.id
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

impl GetOptionId for BoolOptionChangeData {
    fn id(&self) -> &OptionId {
        &self.id
    }
}
