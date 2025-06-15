use std::fmt::Display;

use serde::{Deserialize, Serialize};
use strum_macros::VariantArray;

pub use crate::SystemSummary;
use crate::{CheckResult, DiskInfo, PreInstallConfirmData, ProcessList};

//  Represents the status of a single step in the installation process
#[derive(Serialize, Default, Deserialize, Debug, Clone, PartialEq)]
pub enum DiskoStepStatus {
    #[default]
    Waiting,
    InProgress,
    Done,
    Failed(String),
}

impl Display for DiskoStepStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

// Represents a single step in the installation process
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct DiskoInstallStep {
    pub name: DiskoInstallStepName,
    pub status: DiskoStepStatus,
}

impl std::fmt::Display for DiskoInstallStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.status)
    }
}

// The install steps that are performed by the Disko installer
#[derive(Hash, Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, VariantArray)]
pub enum DiskoInstallStepName {
    Deps,
    Build,
    Disk,
    Mount,
    Copy,
    Bootloader,
}

impl Display for DiskoInstallStepName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl DiskoInstallStepName {
    pub fn description_str(&self) -> &str {
        match self {
            DiskoInstallStepName::Deps => "Fetching Dependencies",
            DiskoInstallStepName::Build => "Building NixOS System",
            DiskoInstallStepName::Disk => "Partitioning & Formatting Disk",
            DiskoInstallStepName::Mount => "Mounting Filesystems",
            DiskoInstallStepName::Copy => "Copying System to Disk",
            DiskoInstallStepName::Bootloader => "Installing Bootloader",
        }
    }
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
    Installing(Vec<DiskoInstallStep>),
    InstallFailed(String),
    InstallSucceeded(Vec<DiskoInstallStep>),
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
    InstallStepUpdate(DiskoInstallStep), // For updating a single step
    InstallLog(String),                  // For sending raw log lines
    Error(String),
}
