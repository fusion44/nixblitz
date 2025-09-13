use iocraft::hooks::State;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionStatus {
    Connecting,
    Connected,
    Disconnected,
}

#[derive(Debug, Clone, strum::Display, PartialEq)]
pub(crate) enum PopupData {
    Option(nixblitz_core::option_data::OptionData),
    Update,
    EngineOffHelp,
}

#[derive(Debug, Clone, strum::Display, Copy, PartialEq, Eq)]
pub(crate) enum Focus {
    AppList,
    OptionList,
    Popup,
}

pub(crate) type SwitchLogsState = Arc<Mutex<State<Vec<String>>>>;
pub(crate) type ShowPopupState = Arc<Mutex<State<bool>>>;
pub(crate) type PopupDataState = Arc<Mutex<State<Option<PopupData>>>>;
pub(crate) type FocusState = Arc<Mutex<State<Focus>>>;
