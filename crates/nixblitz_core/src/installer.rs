use serde::{Deserialize, Serialize};

pub use crate::SystemSummary;
use crate::{CheckResult, DiskInfo, PreInstallConfirmData, ProcessList};

//  Represents the status of a single step in the installation process
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum StepStatus {
    Waiting,
    InProgress,
    Done,
    Failed(String),
}

// Represents a single step in the installation process
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct InstallStep {
    pub id: String,
    pub name: String,
    pub status: StepStatus,
}

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
    Installing(Vec<InstallStep>),
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
    InstallStepUpdate(InstallStep), // For updating a single step
    InstallLog(String),             // For sending raw log lines
    Error(String),
}
