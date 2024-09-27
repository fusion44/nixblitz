use nixblitzlib::apps::SupportedApps;
use serde::{Deserialize, Serialize};
use strum::Display;

use crate::constants::FocusableComponent;

#[derive(Debug, Clone, PartialEq, Eq, Display, Serialize, Deserialize)]
pub enum Action {
    Tick,
    Render,
    Resize(u16, u16),
    Suspend,
    Resume,
    Quit,
    ClearScreen,
    Error(String),
    Help,
    NavAppsTab,
    NavSettingsTab,
    NavActionsTab,
    NavHelpTab,
    NavUp,
    NavDown,
    NavLeft,
    NavRight,
    Enter,
    Esc,
    PageUp,
    PageDown,
    FocusRequest(FocusableComponent),

    /// A new modal is opened
    PushModal,

    /// A modal is closed
    PopModal,

    // App tab specific actions
    AppTabAppSelected(SupportedApps),
}
