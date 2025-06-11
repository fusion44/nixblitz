use log::{info, trace};
use nixblitz_core::{CheckResult, ProcessList, SystemSummary};
use sysinfo::System;

/// Perform the system check and return the results.
pub fn perform_system_check(summary: &SystemSummary) -> CheckResult {
    const MIN_RAM_MB: u64 = 8192;
    const MIN_CPU_CORES: usize = 4;

    let mut issues = Vec::new();

    if summary.total_memory < MIN_RAM_MB {
        issues.push(format!(
            "Insufficient RAM: {} MB found, {} MB required.",
            summary.total_memory, MIN_RAM_MB
        ));
    }

    if summary.cpus.len() < MIN_CPU_CORES {
        issues.push(format!(
            "Insufficient CPU cores: {} found, {} required.",
            summary.cpus.len(),
            MIN_CPU_CORES
        ));
    }

    // TODO: Add other checks

    CheckResult {
        summary: summary.clone(),
        is_compatible: issues.is_empty(),
        issues,
    }
}

/// Get system information using sysinfo
pub fn get_system_info() -> SystemSummary {
    info!("Gathering system information");
    let mut sys = System::new_all();
    sys.refresh_all();
    let system_info = SystemSummary::from(&sys);
    trace!("Complete system information: {:?}", system_info);

    system_info
}

/// Get process information using sysinfo
pub fn get_process_list() -> ProcessList {
    info!("Gathering process list");
    let mut sys = System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

    ProcessList::from(&sys)
}
