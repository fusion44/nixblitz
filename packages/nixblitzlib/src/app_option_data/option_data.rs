use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::apps::SupportedApps;

use super::{
    bool_data::{BoolOptionChangeData, BoolOptionData},
    string_list_data::{StringListOptionChangeData, StringListOptionData},
    text_edit_data::{TextOptionChangeData, TextOptionData},
};

#[derive(Debug, Default, Hash, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OptionId {
    pub app: SupportedApps,
    pub option: String,
}

impl Display for OptionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.app, self.option)
    }
}

impl OptionId {
    pub fn new(app: SupportedApps, option: String) -> Self {
        Self { app, option }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptionData {
    Bool(Box<BoolOptionData>),
    StringList(Box<StringListOptionData>),
    TextEdit(Box<TextOptionData>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptionDataChangeNotification {
    Bool(BoolOptionChangeData),
    StringList(StringListOptionChangeData),
    TextEdit(TextOptionChangeData),
}
