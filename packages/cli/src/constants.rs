pub mod styling;

use serde::{Deserialize, Serialize};
pub use styling::*;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FocusableComponent {
    AppTabList,
    AppTabOptions,
    AppTabHelp,
    SettingsTabOptions,
    #[default]
    None,
}
