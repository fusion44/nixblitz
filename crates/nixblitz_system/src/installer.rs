use std::process::Command;

use log::{debug, info, trace, warn};
use nixblitz_core::{CheckResult, CommandError, DiskInfo, ProcessList, SystemSummary};
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

/// Get disk information using lsblk command with JSON output
pub fn get_disk_info() -> Result<Vec<DiskInfo>, CommandError> {
    debug!("Retrieving disk information using lsblk");

    // Run lsblk with JSON output for reliable parsing
    let cmd = "lsblk";
    let args = ["-b", "-J", "-o", "NAME,SIZE,TYPE,MOUNTPOINTS,RM"];

    info!("Executing command: {} {}", cmd, args.join(" "));

    let output = Command::new("lsblk").args(args).output().map_err(|e| {
        CommandError::SpawnFailed(format!(
            "Failed to execute lsblk command: {} {}\n{}",
            cmd,
            args.join(" "),
            e
        ))
    })?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    trace!("lsblk raw output: {}", output_str);

    let parsed: serde_json::Value = serde_json::from_str(&output_str).map_err(|e| {
        CommandError::SpawnFailed(format!(
            "Error parsing 'lsblk' output:\n Cmd:\n{}\nOutput:\n{}\n",
            format_args!("{} {}", cmd, args.join(" ")),
            e
        ))
    })?;

    let mut disks = Vec::new();

    if let Some(devices) = parsed["blockdevices"].as_array() {
        trace!("Found {} block devices", devices.len());

        for device in devices {
            let name = match device["name"].as_str() {
                Some(n) => n.to_string(),
                None => {
                    debug!("Skipping device with no name");
                    continue;
                }
            };

            let device_type = match device["type"].as_str() {
                Some(t) => t,
                None => {
                    debug!("Skipping device with no type: {}", name);
                    continue;
                }
            };

            if device_type != "disk" {
                debug!("Skipping non-disk device: {} (type: {})", name, device_type);
                continue;
            }

            if !name.starts_with("sd")
                && !name.starts_with("nvme")
                && !name.starts_with("hd")
                && !name.starts_with("vd")
            {
                debug!("Skipping unsupported disk type: {}", name);
                continue;
            }

            let size_bytes = device["size"].as_u64().unwrap_or(0);
            let is_removable = device["rm"].as_u64().unwrap_or(0) == 1;

            let mut mount_points = Vec::new();
            if let Some(mounts) = device["mountpoints"].as_array() {
                for mount in mounts {
                    if let Some(mount_str) = mount.as_str() {
                        if !mount_str.is_empty() && mount_str != "null" {
                            mount_points.push(mount_str.to_string());
                        }
                    }
                }
            }

            let path = format!("/dev/{}", name);

            // Check if the disk is potentially part of a live system
            let is_live_system = mount_points.iter().any(|mp| {
                mp == "/" || mp == "/iso" || mp == "/nix/.ro-store" || mp == "/nix/store"
            });

            if is_live_system {
                warn!(
                    "Detected disk that appears to be part of the live system: {}",
                    path
                );
            }

            trace!(
                "Found disk: {} - {} bytes, removable: {}, live system: {}, mount points: {:?}",
                path, size_bytes, is_removable, is_live_system, mount_points
            );

            disks.push(DiskInfo {
                name,
                path,
                size_bytes,
                mount_points,
                is_removable,
                is_live_system,
            });
        }
    } else {
        trace!("No block devices found in lsblk output");
    }

    trace!("Found {} usable disks", disks.len());
    Ok(disks)
}
