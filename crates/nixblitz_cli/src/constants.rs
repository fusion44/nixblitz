use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FocusableComponent {
    AppTabList,
    AppTabOptions,
    AppTabHelp,
    SettingsTabOptions,
    #[default]
    None,
}
