use serde::{Deserialize, Serialize};

use strum_macros::{Display, VariantArray};

// The switch steps that are performed by the system
#[derive(
    Hash,
    Serialize,
    Deserialize,
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    VariantArray,
    Display,
)]
pub enum SystemConfigSwitchStepName {
    Deps,
    Build,
    Bootloader,
    PostSwitch,
}

// State of the installation, visible to all clients
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Display)]
pub enum SystemState {
    Idle,
    Switching,
    UpdateFailed(String),
    UpdateSucceeded,
}

// Commands from any client to the server
#[derive(Serialize, Deserialize, Debug, Display)]
pub enum SystemClientCommand {
    SwitchConfig,
    DevReset,
    Reboot,
}

// Events from the server to all clients
#[derive(Serialize, Deserialize, Debug, Clone, Display)]
pub enum SystemServerEvent {
    StateChanged(SystemState),
    UpdateLog(String),
    Error(String),
}
