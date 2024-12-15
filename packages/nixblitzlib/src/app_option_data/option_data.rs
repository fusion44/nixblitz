use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::apps::SupportedApps;

use super::{
    bool_data::{BoolOptionChangeData, BoolOptionData},
    password_data::{PasswordOptionChangeData, PasswordOptionData},
    string_list_data::{StringListOptionChangeData, StringListOptionData},
    text_edit_data::{TextOptionChangeData, TextOptionData},
};

/// A trait for obtaining the unique identifier of an option.
pub trait GetOptionId {
    /// Returns a reference to the `OptionId` of the implementing type.
    fn id(&self) -> &OptionId;
}

/// A trait for converting an object into an `OptionId`.
pub trait ToOptionId {
    /// Converts the implementing type into an `OptionId`.
    fn to_option_id(&self) -> OptionId;
}

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
    PasswordEdit(Box<PasswordOptionData>),
}

impl GetOptionId for OptionData {
    fn id(&self) -> &OptionId {
        match self {
            OptionData::Bool(data) => data.id(),
            OptionData::StringList(data) => data.id(),
            OptionData::TextEdit(data) => data.id(),
            OptionData::PasswordEdit(data) => data.id(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptionDataChangeNotification {
    Bool(BoolOptionChangeData),
    StringList(StringListOptionChangeData),
    TextEdit(TextOptionChangeData),
    PasswordEdit(PasswordOptionChangeData),
}

impl GetOptionId for OptionDataChangeNotification {
    fn id(&self) -> &OptionId {
        match self {
            OptionDataChangeNotification::Bool(data) => data.id(),
            OptionDataChangeNotification::StringList(data) => data.id(),
            OptionDataChangeNotification::TextEdit(data) => data.id(),
            OptionDataChangeNotification::PasswordEdit(data) => data.id(),
        }
    }
}
