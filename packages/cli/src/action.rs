use nixblitzlib::{
    app_option_data::option_data::OptionDataChangeNotification, apps::SupportedApps,
};
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

    /// A modal is opened.
    ///
    /// This variant indicates that a modal has been opened.
    /// The `bool` value specifies whether the modal requests exclusive input. For example
    /// if it contains a TextArea and must directly consume all input.
    /// - `true`: The modal has a text area, no action except [Actions::Esc] will be forwarded to
    ///           any components
    /// - `false`: The modal does not have a text area. User input behavior is handleded normally.
    PushModal(bool),

    /// A modal is closed
    /// This variant indicated that the current modal is finished and can be closed.
    /// The `bool` value, if true, specifies whether the modal was canceled or not.
    PopModal(bool),

    /// App tab specific actions
    AppTabAppSelected(SupportedApps),
    AppTabOptionChanged(OptionDataChangeNotification),
}
