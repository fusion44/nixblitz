use serde::{Deserialize, Serialize};

pub use crate::SystemSummary;
use crate::{CheckResult, DiskInfo, PreInstallConfirmData, ProcessList};

// State of the installation, visible to all clients
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum InstallState {
    Idle,
    PerformingCheck,
    SystemCheckCompleted(CheckResult),
    UpdateConfig,
    SelectInstallDisk(Vec<DiskInfo>),
    SelectDiskError(String),
    PreInstallConfirm(PreInstallConfirmData),
    Installing(String),
    InstallFailed(String),
    InstallSucceeded,
}

// Commands from any client to the server
#[derive(Serialize, Deserialize, Debug)]
pub enum ClientCommand {
    PerformSystemCheck,
    GetSystemSummary,
    GetProcessList,
    UpdateConfig,
    UpdateConfigFinished,
    InstallDiskSelected(String),
    StartInstallation,
    DevReset,
}

// Events from the server to all clients
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerEvent {
    StateChanged(InstallState),
    SystemSummaryUpdated(SystemSummary),
    ProcessListUpdated(ProcessList),
    Log(String),
    Error(String),
}
