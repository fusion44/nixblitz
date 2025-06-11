use serde::{Deserialize, Serialize};
use sysinfo::{Cpu, Process, ProcessStatus, System};

/// Summary and the results of the compatibility check.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CheckResult {
    pub summary: SystemSummary,
    pub is_compatible: bool,
    pub issues: Vec<String>,
}

/// A representation of the entire system's state.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SystemSummary {
    pub total_memory: u64,
    pub used_memory: u64,
    pub total_swap: u64,
    pub used_swap: u64,

    pub os_name: String,
    pub os_version: String,
    pub kernel_version: String,
    pub hostname: String,

    pub cpus: Vec<SerializableCpu>,
}

/// A representation of a single CPU.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SerializableCpu {
    pub name: String,
    pub cpu_usage: f32,
    pub frequency: u64,
    pub vendor_id: String,
    pub brand: String,
}

/// A representation of a single Process.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SerializableProcess {
    pub pid: u32,
    pub name: String,
    pub command: Vec<String>,
    pub cpu_usage: f32,
    pub memory: u64,         // in bytes
    pub virtual_memory: u64, // in bytes
    pub status: SerializableProcessStatus,
    pub parent_pid: Option<u32>,
    pub user_id: Option<String>,
    pub start_time: u64, // seconds since Unix epoch
    pub run_time: u64,   // seconds
}

/// A version of the sysinfo::ProcessStatus enum.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum SerializableProcessStatus {
    Idle,
    Run,
    Sleep,
    Stop,
    Zombie,
    Tracing,
    Dead,
    Wakekill,
    Waking,
    Parked,
    LockBlocked,
    UninterruptibleDiskSleep,
    Unknown(u32),
}

impl From<&System> for SystemSummary {
    fn from(sys: &System) -> Self {
        SystemSummary {
            total_memory: sys.total_memory(),
            used_memory: sys.used_memory(),
            total_swap: sys.total_swap(),
            used_swap: sys.used_swap(),
            os_name: System::name().unwrap_or_default(),
            os_version: System::os_version().unwrap_or_default(),
            kernel_version: System::kernel_version().unwrap_or_default(),
            hostname: System::host_name().unwrap_or_default(),
            cpus: sys.cpus().iter().map(|cpu| cpu.into()).collect(),
        }
    }
}

/// Convert from sysinfo::Cpu to our SerializableCpu
impl From<&Cpu> for SerializableCpu {
    fn from(cpu: &Cpu) -> Self {
        SerializableCpu {
            name: cpu.name().to_string(),
            cpu_usage: cpu.cpu_usage(),
            frequency: cpu.frequency(),
            vendor_id: cpu.vendor_id().to_string(),
            brand: cpu.brand().to_string(),
        }
    }
}

/// Convert from sysinfo::Process to our SerializableProcess
impl From<&Process> for SerializableProcess {
    fn from(process: &Process) -> Self {
        SerializableProcess {
            pid: process.pid().as_u32(),
            name: process.name().to_str().unwrap_or_default().to_string(),
            command: process
                .cmd()
                .iter()
                .map(|s| s.to_str().unwrap_or_default().to_string())
                .collect(),
            cpu_usage: process.cpu_usage(),
            memory: process.memory(),
            virtual_memory: process.virtual_memory(),
            status: process.status().into(),
            parent_pid: process.parent().map(|p| p.as_u32()),
            user_id: process.user_id().map(|uid| uid.to_string()),
            start_time: process.start_time(),
            run_time: process.run_time(),
        }
    }
}

// Convert from the sysinfo enum to our serializable enum
impl From<ProcessStatus> for SerializableProcessStatus {
    fn from(status: ProcessStatus) -> Self {
        match status {
            ProcessStatus::Idle => SerializableProcessStatus::Idle,
            ProcessStatus::Run => SerializableProcessStatus::Run,
            ProcessStatus::Sleep => SerializableProcessStatus::Sleep,
            ProcessStatus::Stop => SerializableProcessStatus::Stop,
            ProcessStatus::Zombie => SerializableProcessStatus::Zombie,
            ProcessStatus::Tracing => SerializableProcessStatus::Tracing,
            ProcessStatus::Dead => SerializableProcessStatus::Dead,
            ProcessStatus::Wakekill => SerializableProcessStatus::Wakekill,
            ProcessStatus::Waking => SerializableProcessStatus::Waking,
            ProcessStatus::Parked => SerializableProcessStatus::Parked,
            ProcessStatus::LockBlocked => SerializableProcessStatus::LockBlocked,
            ProcessStatus::UninterruptibleDiskSleep => {
                SerializableProcessStatus::UninterruptibleDiskSleep
            }
            ProcessStatus::Unknown(u) => SerializableProcessStatus::Unknown(u),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProcessList {
    pub processes: Vec<SerializableProcess>,
}

impl From<&System> for ProcessList {
    fn from(sys: &System) -> Self {
        ProcessList {
            processes: sys.processes().values().map(|proc| proc.into()).collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct DiskInfo {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub mount_points: Vec<String>,
    pub is_removable: bool,
    pub is_live_system: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct PreInstallConfirmData {
    pub apps: Vec<String>,
    pub disk: String,
}
