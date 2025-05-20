use std::fmt::Display;

use crate::apps::SupportedApps;
use serde::{Deserialize, Serialize};

use super::{
    bool_data::{BoolOptionChangeData, BoolOptionData},
    net_address_data::{NetAddressOptionChangeData, NetAddressOptionData},
    number_data::{NumberOptionChangeData, NumberOptionData},
    password_data::{PasswordOptionChangeData, PasswordOptionData},
    path_data::{PathOptionChangeData, PathOptionData},
    port_data::{PortOptionChangeData, PortOptionData},
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

/// A trait for converting an object into a Nix-compatible optionally quoted string value.
/// When the value is `None`, the function will return `null`.
pub trait ToNixString {
    /// Converts the implementing type into a Nix-compatible string value.
    fn to_nix_string(&self, quote: bool) -> String;
}

#[derive(Debug, Default, Hash, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OptionId {
    pub app: SupportedApps,
    pub option: String,
}

impl Display for OptionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.app, self.option)
    }
}

impl OptionId {
    pub fn new(app: SupportedApps, option: String) -> Self {
        Self { app, option }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OptionData {
    Bool(Box<BoolOptionData>),
    StringList(Box<StringListOptionData>),
    TextEdit(Box<TextOptionData>),
    Path(Box<PathOptionData>),
    PasswordEdit(Box<PasswordOptionData>),
    NumberEdit(Box<NumberOptionData>),
    NetAddress(Box<NetAddressOptionData>),
    Port(Box<PortOptionData>),
}

/// A trait for marking option data as applied.
///
/// This trait is intended for types that represent option data which can be marked
/// as "applied". Implementers of this trait should provide a mechanism to update
/// the internal state to reflect that the option has been applied to the current system
/// configuration. typically by setting a `applied` flag to `false`.
pub trait ApplicableOptionData {
    /// Marks the option data as applied.
    ///
    /// This method should update the internal state of the implementer to indicate
    /// that the option has been applied. For example, it might set a `applied` flag
    /// to `false` to show that there are no pending changes.
    fn set_applied(&mut self);
}

impl GetOptionId for OptionData {
    fn id(&self) -> &OptionId {
        match self {
            OptionData::Bool(data) => data.id(),
            OptionData::StringList(data) => data.id(),
            OptionData::TextEdit(data) => data.id(),
            OptionData::Path(data) => data.id(),
            OptionData::PasswordEdit(data) => data.id(),
            OptionData::NumberEdit(data) => data.id(),
            OptionData::NetAddress(data) => data.id(),
            OptionData::Port(data) => data.id(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OptionDataChangeNotification {
    Bool(BoolOptionChangeData),
    StringList(StringListOptionChangeData),
    TextEdit(TextOptionChangeData),
    Path(PathOptionChangeData),
    PasswordEdit(PasswordOptionChangeData),
    Number(NumberOptionChangeData),
    NetAddress(NetAddressOptionChangeData),
    Port(PortOptionChangeData),
}

impl GetOptionId for OptionDataChangeNotification {
    fn id(&self) -> &OptionId {
        match self {
            OptionDataChangeNotification::Bool(data) => data.id(),
            OptionDataChangeNotification::StringList(data) => data.id(),
            OptionDataChangeNotification::TextEdit(data) => data.id(),
            OptionDataChangeNotification::Path(data) => data.id(),
            OptionDataChangeNotification::PasswordEdit(data) => data.id(),
            OptionDataChangeNotification::Number(data) => data.id(),
            OptionDataChangeNotification::NetAddress(data) => data.id(),
            OptionDataChangeNotification::Port(data) => data.id(),
        }
    }
}
