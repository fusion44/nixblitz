use error_stack::{Report, Result, ResultExt};
use inquire::{Confirm, Select};
use log::{debug, error, info, warn};
use serde_json;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::{fmt, thread};
use sysinfo::System;

use crate::errors::CliError;

#[derive(Debug)]
enum PostInstallOption {
    Reboot,
    PowerOff,
    ViewLogs,
}

const REBOOT: &str = "Reboot";
const POWER_OFF: &str = "PowerOff";
const VIEW_LOGS: &str = "ViewLogs";

impl fmt::Display for PostInstallOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            PostInstallOption::Reboot => REBOOT,
            PostInstallOption::PowerOff => POWER_OFF,
            PostInstallOption::ViewLogs => VIEW_LOGS,
        };
        write!(f, "{}", s)
    }
}

impl FromStr for PostInstallOption {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, ()> {
        match s {
            REBOOT => Ok(PostInstallOption::Reboot),
            POWER_OFF => Ok(PostInstallOption::PowerOff),
            VIEW_LOGS => Ok(PostInstallOption::ViewLogs),
            _ => Err(()),
        }
    }
}

impl PostInstallOption {
    fn as_str_array() -> [&'static str; 3] {
        [REBOOT, POWER_OFF, VIEW_LOGS]
    }
}

#[derive(Debug)]
enum InstallTarget {
    Local,
    Remote,
}

#[derive(Debug)]
struct SystemRequirements {
    min_ram_mb: u64,
    min_disk_space_gb: u64,
    min_cpu_cores: usize,
}

impl Default for SystemRequirements {
    fn default() -> Self {
        Self {
            min_ram_mb: 8192, // 8GB RAM
            //min_disk_space_gb: 2048, // 2TB storage
            min_disk_space_gb: 100, // for testing
            min_cpu_cores: 4,       // 4 CPU cores
        }
    }
}

#[derive(Debug)]
struct SystemInfo {
    system_type: String,
    cpu_info: String,
    cpu_cores: usize,
    ram_mb: u64,
    disks: Vec<DiskInfo>,
}

#[derive(Debug, Clone)]
struct DiskInfo {
    name: String,
    path: String,
    size_bytes: u64,
    mount_points: Vec<String>,
    is_removable: bool,
    is_live_system: bool,
}

/// Get disk information using lsblk command with JSON output
fn get_disk_info() -> Result<Vec<DiskInfo>, CliError> {
    debug!("Retrieving disk information using lsblk");

    // Run lsblk with JSON output for reliable parsing
    let cmd = "lsblk";
    let args = ["-b", "-J", "-o", "NAME,SIZE,TYPE,MOUNTPOINTS,RM"];

    info!("Executing command: {} {}", cmd, args.join(" "));

    let output = Command::new("lsblk").args(args).output().map_err(|e| {
        error!("Failed to execute lsblk command: {}", e);
        CliError::CommandError(format!("{} {}", cmd, args.join(" ")), e.to_string())
    })?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    debug!("lsblk raw output: {}", output_str);

    // Parse JSON output
    info!("Parsing lsblk JSON output");
    let parsed: serde_json::Value = serde_json::from_str(&output_str).map_err(|e| {
        error!(
            "Error parsing 'lsblk' output:\n Cmd:\n{}\nOutput:\n{}\n",
            format!("{} {}", cmd, args.join(" ")),
            e.to_string()
        );
        CliError::JsonParseError
    })?;

    let mut disks = Vec::new();

    // Process the blockdevices from the JSON output
    if let Some(devices) = parsed["blockdevices"].as_array() {
        info!("Found {} block devices", devices.len());

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

            // Only include disks (not partitions or other types)
            if device_type != "disk" {
                debug!("Skipping non-disk device: {} (type: {})", name, device_type);
                continue;
            }

            // Skip certain device types
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

            // Get mount points
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

            info!(
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
        warn!("No block devices found in lsblk output");
    }

    info!("Found {} usable disks", disks.len());
    Ok(disks)
}

/// Gather system information using sysinfo
fn gather_system_info() -> Result<SystemInfo, CliError> {
    info!("Gathering system information");

    let mut sys = System::new_all();
    debug!("Refreshing system information");
    sys.refresh_all();

    // Get CPU information
    let cpu_info = if !sys.cpus().is_empty() {
        let cpu = &sys.cpus()[0];
        let info = format!("{} ({} MHz)", cpu.name(), cpu.frequency());
        info!("Detected CPU: {}", info);
        info
    } else {
        warn!("Could not detect CPU information");
        "Unknown CPU".to_string()
    };

    // System type
    let system_name = System::name().unwrap_or_else(|| {
        warn!("Could not detect system name");
        "Unknown".to_string()
    });

    let system_os_version = System::os_version().unwrap_or_else(|| {
        warn!("Could not detect OS version");
        "Unknown".to_string()
    });

    let system_type = format!("{} {}", system_name, system_os_version);
    info!("Detected system type: {}", system_type);

    // RAM
    let ram_mb = sys.total_memory() / 1024 / 1024;
    info!("Detected RAM: {} MB", ram_mb);

    // CPU cores
    let cpu_cores = sys.cpus().len();
    info!("Detected CPU cores: {}", cpu_cores);

    // Disks - use our custom function to get better disk info
    info!("Gathering disk information");
    let disks = get_disk_info()?;

    let system_info = SystemInfo {
        system_type,
        cpu_info,
        cpu_cores,
        ram_mb,
        disks,
    };

    debug!("Complete system information: {:?}", system_info);

    Ok(system_info)
}

/// Check if the system meets the minimum requirements
fn check_system_compatibility(
    info: &SystemInfo,
    requirements: &SystemRequirements,
) -> (bool, Vec<String>) {
    info!("Checking system compatibility against requirements");
    debug!("System requirements: {:?}", requirements);

    let mut compatible = true;
    let mut issues = Vec::new();

    // Check RAM
    if info.ram_mb < requirements.min_ram_mb {
        compatible = false;
        let issue = format!(
            "Insufficient RAM: {} MB (minimum: {} MB)",
            info.ram_mb, requirements.min_ram_mb
        );
        warn!("Compatibility issue: {}", issue);
        issues.push(issue);
    } else {
        info!(
            "RAM check passed: {} MB (minimum: {} MB)",
            info.ram_mb, requirements.min_ram_mb
        );
    }

    // Check CPU cores
    if info.cpu_cores < requirements.min_cpu_cores {
        compatible = false;
        let issue = format!(
            "Insufficient CPU cores: {} (minimum: {})",
            info.cpu_cores, requirements.min_cpu_cores
        );
        warn!("Compatibility issue: {}", issue);
        issues.push(issue);
    } else {
        info!(
            "CPU cores check passed: {} cores (minimum: {})",
            info.cpu_cores, requirements.min_cpu_cores
        );
    }

    // Check if there are any disks with enough space
    let min_disk_bytes = requirements.min_disk_space_gb * 1024 * 1024 * 1024;
    let largest_disk_bytes = info.disks.iter().map(|d| d.size_bytes).max().unwrap_or(0);
    let has_sufficient_disk = info
        .disks
        .iter()
        .any(|disk| disk.size_bytes >= min_disk_bytes);

    if !has_sufficient_disk {
        compatible = false;
        let issue = format!(
            "No disk with sufficient space found. Minimum required: {} GB (largest available: {} GB)",
            requirements.min_disk_space_gb,
            largest_disk_bytes / 1024 / 1024 / 1024
        );
        warn!("Compatibility issue: {}", issue);
        issues.push(issue);
    } else {
        info!(
            "Disk space check passed: found at least one disk with >= {} GB",
            requirements.min_disk_space_gb
        );
    }

    if compatible {
        info!("System meets all compatibility requirements");
    } else {
        warn!(
            "System does not meet all compatibility requirements. Found {} issues",
            issues.len()
        );
    }

    (compatible, issues)
}

/// Implementation of the install wizard
pub fn install_wizard(work_dir: &Path) -> Result<(), CliError> {
    info!("Starting NixBlitz installation wizard");
    info!("Working directory: {:?}", work_dir);

    println!("Welcome to the NixBlitz Installation Wizard");
    println!("===========================================\n");

    // Step 1: Ask whether to install locally or remotely
    info!("Step 1: Requesting installation target selection (local/remote)");
    let install_options = vec![
        "On this machine (local installation)",
        "On a remote machine",
    ];

    let target_choice = Select::new("Where would you like to install NixBlitz?", install_options)
        .prompt()
        .map_err(|e| {
            error!("Failed to get installation target selection: {:?}", e);
            CliError::ArgumentError
        })?;

    let install_target = if target_choice == "On this machine (local installation)" {
        info!("User selected local installation target");
        InstallTarget::Local
    } else {
        info!("User selected remote installation target");
        InstallTarget::Remote
    };

    match install_target {
        InstallTarget::Remote => {
            info!("Remote installation requested but not implemented yet");
            println!("\nRemote installation is not yet implemented.");
            println!("This feature will be available in a future release.");
            println!("Please choose local installation for now.");
            return Ok(());
        }
        InstallTarget::Local => {
            info!("Proceeding with local installation");
            println!("\nPreparing for local installation...");
            // Continue with local installation
        }
    }

    // Step 2: Analyze the local system
    info!("Step 2: Analyzing system information");
    println!("\n🔍 Analyzing your system...");
    let system_info = gather_system_info().map_err(|e| {
        error!("Failed to gather system information: {:?}", e);
        e
    })?;

    info!("System information gathered successfully");
    debug!("System info: {:?}", system_info);
    info!("System type: {}", system_info.system_type);
    info!("CPU: {}", system_info.cpu_info);
    info!("CPU Cores: {}", system_info.cpu_cores);
    info!("RAM: {} MB", system_info.ram_mb);
    info!("Found {} disk(s)", system_info.disks.len());

    for disk in &system_info.disks {
        let size_gb = disk.size_bytes / 1024 / 1024 / 1024;
        info!(
            "Disk: {} ({} GB), removable: {}, live system: {}, mount points: {:?}",
            disk.path, size_gb, disk.is_removable, disk.is_live_system, disk.mount_points
        );
    }

    // Display system information
    println!("\n🖥️ System Information:");
    println!("------------------------");
    println!("🖥️ System: {}", system_info.system_type);
    println!("🧠 CPU: {}", system_info.cpu_info);
    println!("🧩 CPU Cores: {}", system_info.cpu_cores);
    println!("💾 RAM: {} MB", system_info.ram_mb);
    println!("\n💽 Available Disks:");

    for (i, disk) in system_info.disks.iter().enumerate() {
        let size_gb = disk.size_bytes / 1024 / 1024 / 1024;
        let mount_info = if disk.mount_points.is_empty() {
            "not mounted".to_string()
        } else {
            format!("mounted at: {}", disk.mount_points.join(", "))
        };

        let removable_tag = if disk.is_removable {
            " [removable]"
        } else {
            ""
        };
        let live_system_tag = if disk.is_live_system {
            " [live system]"
        } else {
            ""
        };

        println!(
            "  {}. {} ({} GB, {}){}{}",
            i + 1,
            disk.path,
            size_gb,
            mount_info,
            removable_tag,
            live_system_tag
        );
    }

    // Step 3: Check system compatibility
    info!("Step 3: Checking system compatibility against requirements");
    let requirements = SystemRequirements::default();
    info!("System requirements: {:?}", requirements);

    let (compatible, issues) = check_system_compatibility(&system_info, &requirements);

    if !compatible {
        warn!("System does not meet minimum requirements");
        for issue in &issues {
            warn!("Compatibility issue: {}", issue);
        }

        println!("\n⚠️ Your system does not meet the minimum requirements:");
        for issue in &issues {
            println!("  - {}", issue);
        }

        info!("Asking user whether to continue despite compatibility issues");
        let continue_anyway = Confirm::new("Do you want to continue anyway? (Not recommended)")
            .with_default(false)
            .prompt()
            .map_err(|e| {
                error!("Failed to get user confirmation: {:?}", e);
                CliError::ArgumentError
            })?;

        if !continue_anyway {
            info!("User chose to abort installation due to compatibility issues");
            println!("Installation aborted.");
            return Ok(());
        }

        info!("User chose to continue despite compatibility issues");
        println!("\nContinuing installation despite system limitations...");
    } else {
        info!("System meets all minimum requirements");
        println!("\n✅ Your system meets all minimum requirements!");
    }

    // Step 4: Let user select installation disk
    info!("Step 4: Requesting disk selection for installation");

    if system_info.disks.is_empty() {
        error!("No suitable disks detected, cannot proceed with installation");
        println!("\n❌ No suitable disks detected. Installation cannot proceed.");
        return Ok(());
    }

    // Create a list of disk descriptions for the user to choose from
    let mut disk_options = Vec::new();
    for disk in &system_info.disks {
        let size_gb = disk.size_bytes / 1024 / 1024 / 1024;
        let mount_info = if disk.mount_points.is_empty() {
            "not mounted".to_string()
        } else {
            format!("mounted at: {}", disk.mount_points.join(", "))
        };

        let removable_info = if disk.is_removable {
            " [REMOVABLE]"
        } else {
            ""
        };

        let live_system_warning = if disk.is_live_system {
            " [CAUTION: May be part of live system]"
        } else {
            ""
        };

        let recommended = if !disk.is_live_system && !disk.is_removable && size_gb >= 32 {
            " [RECOMMENDED]"
        } else {
            ""
        };

        let description = format!(
            "{} ({} GB, {}){}{}{}",
            disk.path, size_gb, mount_info, removable_info, live_system_warning, recommended
        );

        disk_options.push(description);
    }

    info!("Displaying disk selection options to user");
    debug!("Disk options: {:?}", disk_options);
    println!("\nIMPORTANT: The selected disk will be COMPLETELY ERASED!");

    let selected_disk_desc = Select::new(
        "Please select a disk to install NixBlitz on:",
        disk_options.clone(),
    )
    .with_help_message("⚠️ WARNING: Selected disk will be completely erased!")
    .prompt()
    .map_err(|e| {
        error!("Failed to get disk selection from user: {:?}", e);
        CliError::ArgumentError
    })?;

    // Find which disk was selected
    let selected_index = disk_options
        .iter()
        .position(|desc| desc == &selected_disk_desc)
        .unwrap();

    let selected_disk = if selected_index < system_info.disks.len() {
        system_info.disks[selected_index].clone()
    } else {
        error!("Selected index is out of bounds");
        return Err(Report::new(CliError::ArgumentError));
    };
    info!(
        "User selected disk: {} [name: {}]",
        selected_disk.path, selected_disk.name
    );
    debug!("Selected disk details: {:?}", selected_disk);

    // Step 5: Disk erasure warning and confirmation
    info!("Step 5: Asking for disk erasure confirmation");
    println!("\n⚠️ WARNING ⚠️");
    println!("The following disk will be COMPLETELY ERASED during installation:");

    let size_gb = selected_disk.size_bytes / 1024 / 1024 / 1024;
    let mount_info = if selected_disk.mount_points.is_empty() {
        "not mounted".to_string()
    } else {
        format!("mounted at: {}", selected_disk.mount_points.join(", "))
    };

    println!("  {} ({} GB, {})", selected_disk.path, size_gb, mount_info);

    if selected_disk.is_live_system {
        warn!(
            "User selected a disk that appears to be part of the live system: {}",
            selected_disk.path
        );
        println!("\n⚠️ CAUTION: This disk appears to be part of the live system you're currently running!");
        println!("Installing to this disk may cause your current system to become unstable.");
    }

    println!("\nALL DATA on this disk will be PERMANENTLY LOST!");

    let proceed = Confirm::new("Are you sure you want to continue?")
        .with_default(false)
        .with_help_message("This action cannot be undone!")
        .prompt()
        .map_err(|e| {
            error!("Failed to get confirmation for disk erasure: {:?}", e);
            CliError::ArgumentError
        })?;

    if !proceed {
        info!("User aborted installation at disk erasure confirmation");
        println!("Installation aborted.");
        return Ok(());
    }

    // Step 6: Installation
    info!(
        "Step 6: Proceeding with installation on disk: {}",
        selected_disk.path
    );

    println!("Launching NixBlitz configuration tool...");

    println!("Working directory: {}", work_dir.to_string_lossy());

    let cmd = Command::new("nixblitz")
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .arg("tui")
        .arg("-w")
        .arg(work_dir)
        .spawn();

    match cmd {
        Ok(mut child) => {
            let status = child.wait().change_context(CliError::Unknown)?;
            println!("\nTUI finished with status: {}.", status);
            std::thread::sleep(std::time::Duration::from_secs(1));

            match build_system_streaming(work_dir, "nixblitzx86vm") {
                Ok(()) => {}
                Err(e) => {
                    eprintln!("\n❌ System build function failed: {}", e);
                    eprintln!("\n--- Full Collected Output (from error) ---");
                    error!("{}", e);
                    eprintln!("--- End of Output ---");

                    std::process::exit(1);
                }
            }

            match install_system_streaming(work_dir, "nixblitzx86vm", &selected_disk.path) {
                Ok(()) => {}
                Err(e) => {
                    eprintln!("\n❌ System build function failed: {}", e);
                    eprintln!("\n--- Full Collected Output (from error) ---");
                    error!("{}", e);
                    eprintln!("--- End of Output ---");

                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("\nError launching TUI: {}", e);
            eprintln!("Please ensure 'nixblitz' is installed and in your PATH.",);
            std::thread::sleep(std::time::Duration::from_secs(3));
        }
    }

    // Step 7: Sync config files
    let res = sync_config(work_dir, &selected_disk.path);
    match res {
        Ok(()) => {
            println!("\n✅ System config copied successfully");
        }
        Err(e) => {
            eprintln!("\n❌ System copy failed: {}", e);
            eprintln!("\n--- Full Collected Output (from error) ---");
            error!("{}", e);
            eprintln!("--- End of Output ---");

            std::process::exit(1);
        }
    }
    println!("If there was no error, you may now reboot your system.\n\n");

    show_post_install_choice()?;

    info!("Installation wizard completed.");
    Ok(())
}

const ERROR_PATTERNS: &[&str] = &[
    "error:",
    "Build failed",
    "killing builder process",
    "undefined variable",
    "attribute not found",
];

fn show_post_install_choice() -> Result<(), CliError> {
    let post_install_choice = Select::new(
        "What do you want to do now?",
        PostInstallOption::as_str_array().to_vec(),
    )
    .prompt()
    .map_err(|e| {
        error!("Failed to get post install option selection: {:?}", e);
        CliError::ArgumentError
    })?;

    match PostInstallOption::from_str(post_install_choice) {
        Ok(PostInstallOption::Reboot) => {
            info!("User selected to reboot the system");
            reboot_system()?;
        }
        Ok(PostInstallOption::PowerOff) => {
            info!("User selected to poweroff the system");
            poweroff_system()?;
        }
        Ok(PostInstallOption::ViewLogs) => {
            info!("User selected to view logs");
            show_log()?;
            show_post_install_choice()?;
        }

        Err(_) => {
            error!("Invalid post install option {}", post_install_choice);
            return Err(Report::new(CliError::ArgumentError));
        }
    }

    Ok(())
}

/// Runs the Nix build command for the specified system configuration, streaming its output.
///
/// # Arguments
///
/// * `work_dir` - The path to the directory containing the flake.nix (e.g., /home/nixos/config/src).
/// * `nixos_config_name` - The name of the nixosConfiguration (e.g., "nixblitzx86vm").
///
/// # Returns
///
/// * `Ok(())` - If the build completes successfully without known error patterns.
/// * `Err(CliError)` - If the command fails to execute, exits with non-zero status,
///   or its streamed output contains defined error patterns.
fn build_system_streaming(work_dir: &Path, nixos_config_name: &str) -> Result<(), CliError> {
    let nix_target = format!(
        "{}/src#nixosConfigurations.{}.config.system.build.toplevel",
        work_dir.to_string_lossy(),
        nixos_config_name
    );

    let sudo_args = &["nix", "build", "--no-update-lock-file", &nix_target];

    println!("──────────────────────────────────────────────────────────────");
    println!("🚀 Starting system build: {}", nixos_config_name);
    println!("🔧 Running command via sudo:");
    println!("   sudo {}", sudo_args.join(" "));
    println!("──────────────────────────────────────────────────────────────");
    io::stdout()
        .flush()
        .change_context(CliError::IoError("Unable to flush stdout".into()))?;

    let proceed = Confirm::new("Are you sure you want to continue?")
        .with_default(false)
        .with_help_message("The actual installation will be done after the build.")
        .prompt()
        .map_err(|e| {
            error!("Failed to get confirmation for disk erasure: {:?}", e);
            CliError::ArgumentError
        })?;

    if !proceed {
        info!("User aborted installation at disk erasure confirmation");
        println!("Installation aborted.");
        return Err(Report::new(CliError::UserAbort));
    }

    let mut child = Command::new("sudo")
        .args(sudo_args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| {
            if e.kind() == io::ErrorKind::NotFound {
                CliError::BuildExecutionFailed(
                    "'sudo' command not found. Is it installed and in PATH?".to_string(),
                )
            } else {
                CliError::IoError(e.to_string())
            }
        })?;

    // Signals if an error pattern was detected in one of the streams
    let error_pattern_detected = Arc::new(AtomicBool::new(false));
    let mut thread_handles = Vec::new();
    let collected_stdout_arc = Arc::new(Mutex::new(String::new()));
    let collected_stderr_arc = Arc::new(Mutex::new(String::new()));

    if let Some(stdout_pipe) = child.stdout.take() {
        let error_flag_clone = Arc::clone(&error_pattern_detected);
        let stdout_collector_clone = Arc::clone(&collected_stdout_arc);
        let stdout_handle = thread::spawn(move || {
            let reader = BufReader::new(stdout_pipe);
            for line_result in reader.lines() {
                match line_result {
                    Ok(line) => {
                        println!("{}", line);
                        if let Ok(mut guard) = stdout_collector_clone.lock() {
                            guard.push_str(&line);
                            guard.push('\n');
                        }
                        for pattern in ERROR_PATTERNS {
                            if line.contains(pattern) {
                                eprintln!(">> ERROR DETECTED (stdout): {}", line);
                                error_flag_clone.store(true, Ordering::SeqCst);
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error reading stdout pipe: {}", e);
                        error_flag_clone.store(true, Ordering::SeqCst);
                        break;
                    }
                }
            }
        });
        thread_handles.push(stdout_handle);
    } else {
        eprintln!("Warning: Could not capture stdout pipe.");
    }

    if let Some(stderr_pipe) = child.stderr.take() {
        let error_flag_clone = Arc::clone(&error_pattern_detected);
        let stderr_collector_clone = Arc::clone(&collected_stderr_arc);
        let stderr_handle = thread::spawn(move || {
            let reader = BufReader::new(stderr_pipe);
            for line_result in reader.lines() {
                match line_result {
                    Ok(line) => {
                        eprintln!("{}", line);
                        if let Ok(mut guard) = stderr_collector_clone.lock() {
                            guard.push_str(&line);
                            guard.push('\n');
                        }
                        for pattern in ERROR_PATTERNS {
                            if line.contains(pattern) {
                                eprintln!(">> ERROR DETECTED (stderr): {}", line);
                                error_flag_clone.store(true, Ordering::SeqCst);
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error reading stderr pipe: {}", e);
                        error_flag_clone.store(true, Ordering::SeqCst);
                        break;
                    }
                }
            }
        });
        thread_handles.push(stderr_handle);
    } else {
        eprintln!("Warning: Could not capture stderr pipe.");
    }

    let final_status = child
        .wait()
        .change_context(CliError::CommandError("".into(), "Unknown".into()))?;

    for handle in thread_handles {
        if let Err(e) = handle.join() {
            eprintln!("Error joining a reader thread: {:?}", e);
            error_pattern_detected.store(true, Ordering::SeqCst);
        }
    }

    let exit_code_failed = !final_status.success();
    let pattern_detected = error_pattern_detected.load(Ordering::SeqCst);

    println!("──────────────────────────────────────────────────────────────");

    if exit_code_failed || pattern_detected {
        let reason = if exit_code_failed {
            format!("Build command exited with status: {}", final_status)
        } else {
            "Detected error pattern in streamed output".to_string()
        };
        eprintln!("❌ Build failed! {}", reason);

        let final_stdout = collected_stdout_arc
            .lock()
            .map_err(|e| CliError::LockError(e.to_string()))?
            .clone();
        let final_stderr = collected_stderr_arc
            .lock()
            .map_err(|e| CliError::LockError(e.to_string()))?
            .clone();
        let final_combined_output = format!(
            "--- STDOUT ---\n{}\n--- STDERR ---\n{}",
            final_stdout, final_stderr
        );

        Err(Report::new(CliError::BuildExecutionFailed(
            final_combined_output,
        )))
    } else {
        println!("✅ Build command completed successfully.");
        let final_stdout = collected_stdout_arc
            .lock()
            .map_err(|e| CliError::LockError(e.to_string()))?
            .clone();
        if let Some(result_path) = final_stdout
            .lines()
            .filter(|l| !l.trim().is_empty())
            .next_back()
        {
            println!("Build result path: {}", result_path.trim());
        }
        println!("──────────────────────────────────────────────────────────────");
        Ok(())
    }
}

fn show_log() -> Result<(), CliError> {
    let args = &["/home/nixos/.local/state/nixblitz-cli/nixblitz.log"];
    let status = Command::new("bat").args(args).status().map_err(|e| {
        error!("Failed to show logs: {}", e);
        CliError::CommandError(
            format!("Failed to run command: bat {:?}", args),
            e.to_string(),
        )
    })?;

    if !status.success() {
        error!("Failed to reboot system. Status: {}", status);
        return Err(Report::new(CliError::CommandError(
            "Reboot failed.".to_string(),
            status.to_string(),
        )));
    }

    Ok(())
}

/// Attempts to reboot the system
fn reboot_system() -> Result<(), CliError> {
    println!(
        "\n\n--------------------------------------------------------------------------------"
    );
    println!("Rebooting system...");
    println!("--------------------------------------------------------------------------------");

    let args = &["systemctl", "reboot"];
    let status = Command::new("sudo").args(args).status().map_err(|e| {
        error!("Failed to reboot system: {}", e);
        CliError::CommandError(
            format!("Failed to run command: sudo {:?}", args),
            e.to_string(),
        )
    })?;

    if !status.success() {
        error!("Failed to reboot system. Status: {}", status);
        return Err(Report::new(CliError::CommandError(
            "Reboot failed.".to_string(),
            status.to_string(),
        )));
    }

    Ok(())
}

/// Attempts to poweroff the system
fn poweroff_system() -> Result<(), CliError> {
    println!(
        "\n\n--------------------------------------------------------------------------------"
    );
    println!("Powering off system...");
    println!("--------------------------------------------------------------------------------");
    let args = &["systemctl", "poweroff"];
    let status = Command::new("sudo").args(args).status().map_err(|e| {
        error!("Failed to poweroff system: {}", e);
        CliError::CommandError(
            format!("Failed to run command: sudo {:?}", args),
            e.to_string(),
        )
    })?;

    if !status.success() {
        error!("Failed to poweroff system. Status: {}", status);
        return Err(Report::new(CliError::CommandError(
            "Poweroff failed.".to_string(),
            status.to_string(),
        )));
    }

    Ok(())
}

/// Attempts to sync the configuration from the work directory to the selected disk
fn sync_config(work_dir: &Path, selected_disk: &str) -> Result<(), CliError> {
    let post = if selected_disk.starts_with("/dev/nvme") {
        // nvme0n1
        // ├─nvme0n1p1
        // ├─nvme0n1p2
        "p3"
    } else {
        // sda
        // └─sda1
        "3"
    };
    let disk = format!("{}{}", selected_disk, post).to_owned();
    let mkdir_args = &["mkdir", "-p", "/mnt/data"];
    let mount_args = &["mount", &disk, "/mnt/data"];
    let rsync_args = &[
        "rsync",
        "-av",
        "--delete",
        &work_dir.to_string_lossy(),
        "/mnt/data/",
    ];
    let chown_args = &["chown", "-R", "1000:100", "/mnt/data/config"];
    println!("──────────────────────────────────────────────────────────────");
    println!("🚀 Copying system config to system");
    println!("🔧 Running the following commands:");
    println!("     sudo {}", mkdir_args.join(" "));
    println!("     sudo {}", mount_args.join(" "));
    println!("     sudo {}", rsync_args.join(" "));
    println!("     sudo {}", chown_args.join(" "));
    println!("──────────────────────────────────────────────────────────────");

    let status = Command::new("sudo")
        .args(mkdir_args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|e| {
            error!("Failed to make data directory: {}", e);
            CliError::CommandError(
                format!("Failed to run command: sudo {:?}", mkdir_args),
                e.to_string(),
            )
        })?;

    if !status.success() {
        error!("Failed to make data directory. Status: {}", status);
        return Err(Report::new(CliError::CommandError(
            "Make data directory failed.".to_string(),
            status.to_string(),
        )));
    }

    let status = Command::new("sudo")
        .args(mount_args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|e| {
            error!("Failed to mount device: {}", e);
            CliError::CommandError(
                format!("Failed to run command: sudo {:?}", mount_args),
                e.to_string(),
            )
        })?;

    if !status.success() {
        error!("Failed to mount device. Status: {}", status);
        return Err(Report::new(CliError::CommandError(
            "Mount failed.".to_string(),
            status.to_string(),
        )));
    }

    let status = Command::new("sudo")
        .args(rsync_args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|e| {
            error!("Failed to sync config: {}", e);
            CliError::CommandError(
                format!("Failed to run command: sudo {:?}", rsync_args),
                e.to_string(),
            )
        })?;

    if !status.success() {
        error!("Failed to sync config. Status: {}", status);
        return Err(Report::new(CliError::CommandError(
            "Sync failed.".to_string(),
            status.to_string(),
        )));
    }

    let status = Command::new("sudo")
        .args(chown_args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|e| {
            error!("Failed to change config folder ownership: {}", e);
            CliError::CommandError(
                format!("Failed to run command: sudo {:?}", chown_args),
                e.to_string(),
            )
        })?;

    if !status.success() {
        error!(
            "Failed to change config folder ownership. Status: {}",
            status
        );
        return Err(Report::new(CliError::CommandError(
            "Chown failed.".to_string(),
            status.to_string(),
        )));
    }

    Ok(())
}

/// Runs the disko-install command for the specified system configuration, streaming its output.
///
/// # Arguments
///
/// * `work_dir` - The path to the directory containing the flake.nix (e.g., /home/nixos/config/src).
/// * `nixos_config_name` - The name of the nixosConfiguration (e.g., "nixblitzx86vm").
/// * `disk` - The name of the disk device to install the system on.
///
/// # Returns
///
/// * `Ok(())` - If the build completes successfully without known error patterns.
/// * `Err(CliError)` - If the command fails to execute, exits with non-zero status,
///   or its streamed output contains defined error patterns.
fn install_system_streaming(
    work_dir: &Path,
    nixos_config_name: &str,
    disk: &str,
) -> Result<(), CliError> {
    let nix_target = format!("{}/src#{}", work_dir.to_string_lossy(), nixos_config_name);

    let sudo_args = &[
        "disko-install",
        "--flake",
        &nix_target,
        "--disk",
        "main",
        disk,
    ];

    println!("──────────────────────────────────────────────────────────────");
    println!("🚀 Starting system install: {}", nixos_config_name);
    println!("🔧 Running command via sudo:");
    println!("   sudo {}", sudo_args.join(" "));
    println!("──────────────────────────────────────────────────────────────");
    io::stdout()
        .flush()
        .change_context(CliError::IoError("Unable to flush stdout".into()))?;

    let proceed = Confirm::new("Are you sure you want to continue?")
        .with_default(false)
        .with_help_message("🚨 This action cannot be undone and will erase the disk! 🚨")
        .prompt()
        .map_err(|e| {
            error!("Failed to get confirmation for disk erasure: {:?}", e);
            CliError::UserAbort
        })?;

    if !proceed {
        info!("User aborted installation at disk erasure confirmation");
        println!("Installation aborted.");
        return Err(Report::new(CliError::UserAbort));
    }

    let mut child = Command::new("sudo")
        .args(sudo_args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| {
            if e.kind() == io::ErrorKind::NotFound {
                CliError::InstallExecutionFailed(
                    "'sudo' command not found. Is it installed and in PATH?".to_string(),
                )
            } else {
                CliError::IoError(e.to_string())
            }
        })?;

    let error_pattern_detected = Arc::new(AtomicBool::new(false));
    let mut thread_handles = Vec::new();
    let collected_stdout_arc = Arc::new(Mutex::new(String::new()));
    let collected_stderr_arc = Arc::new(Mutex::new(String::new()));

    if let Some(stdout_pipe) = child.stdout.take() {
        let error_flag_clone = Arc::clone(&error_pattern_detected);
        let stdout_collector_clone = Arc::clone(&collected_stdout_arc);
        let stdout_handle = thread::spawn(move || {
            let reader = BufReader::new(stdout_pipe);
            for line_result in reader.lines() {
                match line_result {
                    Ok(line) => {
                        println!("{}", line);
                        if let Ok(mut guard) = stdout_collector_clone.lock() {
                            guard.push_str(&line);
                            guard.push('\n');
                        }
                        for pattern in ERROR_PATTERNS {
                            if line.contains(pattern) {
                                eprintln!(">> ERROR DETECTED (stdout): {}", line);
                                error_flag_clone.store(true, Ordering::SeqCst);
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error reading stdout pipe: {}", e);
                        error_flag_clone.store(true, Ordering::SeqCst);
                        break;
                    }
                }
            }
        });
        thread_handles.push(stdout_handle);
    } else {
        eprintln!("Warning: Could not capture stdout pipe.");
    }

    if let Some(stderr_pipe) = child.stderr.take() {
        let error_flag_clone = Arc::clone(&error_pattern_detected);
        let stderr_collector_clone = Arc::clone(&collected_stderr_arc);
        let stderr_handle = thread::spawn(move || {
            let reader = BufReader::new(stderr_pipe);
            for line_result in reader.lines() {
                match line_result {
                    Ok(line) => {
                        eprintln!("{}", line);
                        if let Ok(mut guard) = stderr_collector_clone.lock() {
                            guard.push_str(&line);
                            guard.push('\n');
                        }
                        for pattern in ERROR_PATTERNS {
                            if line.contains(pattern) {
                                eprintln!(">> ERROR DETECTED (stderr): {}", line);
                                error_flag_clone.store(true, Ordering::SeqCst);
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error reading stderr pipe: {}", e);
                        error_flag_clone.store(true, Ordering::SeqCst);
                        break;
                    }
                }
            }
        });
        thread_handles.push(stderr_handle);
    } else {
        eprintln!("Warning: Could not capture stderr pipe.");
    }

    let final_status = child
        .wait()
        .change_context(CliError::CommandError("".into(), "Unknown".into()))?;

    for handle in thread_handles {
        if let Err(e) = handle.join() {
            eprintln!("Error joining a reader thread: {:?}", e);
            error_pattern_detected.store(true, Ordering::SeqCst);
        }
    }

    let exit_code_failed = !final_status.success();
    let pattern_detected = error_pattern_detected.load(Ordering::SeqCst);

    println!("──────────────────────────────────────────────────────────────");

    if exit_code_failed || pattern_detected {
        let reason = if exit_code_failed {
            format!("Install command exited with status: {}", final_status)
        } else {
            "Detected error pattern in streamed output".to_string()
        };
        eprintln!("❌ Install failed! {}", reason);

        let final_stdout = collected_stdout_arc
            .lock()
            .map_err(|e| CliError::LockError(e.to_string()))?
            .clone();
        let final_stderr = collected_stderr_arc
            .lock()
            .map_err(|e| CliError::LockError(e.to_string()))?
            .clone();
        let final_combined_output = format!(
            "--- STDOUT ---\n{}\n--- STDERR ---\n{}",
            final_stdout, final_stderr
        );

        Err(Report::new(CliError::InstallExecutionFailed(
            final_combined_output,
        )))
    } else {
        println!("✅ Install command completed successfully.");
        println!("──────────────────────────────────────────────────────────────");
        Ok(())
    }
}
