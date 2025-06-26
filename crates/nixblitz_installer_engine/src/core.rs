use crate::engine::{EngineState, SharedInstallState};
use error_stack::{FutureExt, Report, Result, ResultExt};
use log::{debug, info};
use nixblitz_core::{
    DiskoInstallStepName, DiskoStepStatus, InstallError, InstallServerEvent, InstallState,
};
use nixblitz_system::utils::exec_simple_command;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::broadcast;

pub async fn install_system(
    state_arc: SharedInstallState,
    sender: broadcast::Sender<InstallServerEvent>,
    work_dir: &String,
    nixos_config_name: &String,
    disk: &str,
) -> error_stack::Result<(), InstallError> {
    let flake_target = format!("{}/src#{}", work_dir, nixos_config_name);

    let args = &[
        "disko-install",
        "--flake",
        &flake_target,
        "--disk",
        "main",
        disk,
    ];

    let cmd_str = format!("sudo {}", args.join(" "));
    info!("──────────────────────────────────────────────────────────────");
    info!("🚀 Starting system install: {}", nixos_config_name);
    info!("🔧 Running command via sudo:");
    info!("   sudo {}", args.join(" "));
    info!("──────────────────────────────────────────────────────────────");

    let mut child = match Command::new("sudo")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(e) => {
            let mut state = state_arc.lock().await;
            let error_msg = format!("Failed to spawn command: '{}'\nError:\n{}", cmd_str, e);
            state.install_state = InstallState::InstallFailed(error_msg);
            let _ = sender.send(InstallServerEvent::StateChanged(
                state.install_state.clone(),
            ));

            return Err(Report::new(InstallError::CommandError(
                cmd_str,
                e.to_string(),
            )));
        }
    };

    let stdout = child.stdout.take().expect("Failed to capture stdout");
    let stderr = child.stderr.take().expect("Failed to capture stderr");

    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();

    let mut current_step_name = DiskoInstallStepName::Deps;

    loop {
        tokio::select! {
            result = stdout_reader.next_line() => {
                match result {
                    Ok(Some(line)) => {
                        info!("[STDOUT]: {}", line);
                        let _ = sender.send(InstallServerEvent::InstallLog(line.clone()));
                        current_step_name = parse_log_and_update_state(&state_arc, &sender, &line, current_step_name).await;
                    },
                    _ => break,
                }
            },
            result = stderr_reader.next_line() => {
                match result {
                    Ok(Some(line)) => {
                        info!("[STDERR]: {}", line);
                        let _ = sender.send(InstallServerEvent::InstallLog(format!("[STDERR] {}", line)));
                        current_step_name = parse_log_and_update_state(&state_arc, &sender, &line, current_step_name).await;
                    },
                    _ => break,
                }
            },
        }
    }

    let status = match child.wait().await {
        Ok(status) => status,
        Err(e) => {
            let mut state = state_arc.lock().await;
            let error_msg = format!("Failed to wait for installer command: {}", e);
            state.install_state = InstallState::InstallFailed(error_msg);
            let _ = sender.send(InstallServerEvent::StateChanged(
                state.install_state.clone(),
            ));
            return Err(Report::new(InstallError::CommandError(
                cmd_str,
                e.to_string(),
            )));
        }
    };

    if status.success() {
        info!("Installation succeeded.");
        let mut state = state_arc.lock().await;
        update_and_send_disko_status(
            &mut state,
            &sender,
            &current_step_name,
            DiskoStepStatus::Done,
        );
        let final_steps = state.disko_install_steps.clone();
        state.install_state = InstallState::InstallSucceeded(final_steps);
    } else {
        let msg = format!("Installation failed with exit code: {}", status);
        let mut state = state_arc.lock().await;
        update_and_send_disko_status(
            &mut state,
            &sender,
            &current_step_name,
            DiskoStepStatus::Failed(msg.clone()),
        );
        state.install_state = InstallState::InstallFailed(msg.clone());
        return Err(Report::new(InstallError::CommandError(cmd_str, msg)));
    }

    Ok(())
}

/// A helper to parse a log line and update the state if a landmark is found.
async fn parse_log_and_update_state(
    state_arc: &SharedInstallState,
    sender: &broadcast::Sender<InstallServerEvent>,
    line: &str,
    current_step: DiskoInstallStepName,
) -> DiskoInstallStepName {
    let mut next_step = None;

    if line.contains("unpacking 'github:") && current_step < DiskoInstallStepName::Deps {
        next_step = Some(DiskoInstallStepName::Deps);
    } else if line.contains("derivations will be built")
        && current_step < DiskoInstallStepName::Build
    {
        next_step = Some(DiskoInstallStepName::Build);
    } else if line.contains("sgdisk") && current_step < DiskoInstallStepName::Disk {
        next_step = Some(DiskoInstallStepName::Disk);
    } else if line.contains("mount /dev/disk") && current_step < DiskoInstallStepName::Mount {
        next_step = Some(DiskoInstallStepName::Mount);
    } else if line.contains("Copying store paths") && current_step < DiskoInstallStepName::Copy {
        next_step = Some(DiskoInstallStepName::Copy);
    } else if line.contains("installing the boot loader")
        && current_step < DiskoInstallStepName::Bootloader
    {
        next_step = Some(DiskoInstallStepName::Bootloader);
    }

    if let Some(step_name) = next_step {
        let mut state = state_arc.lock().await;
        // Mark the previous step as Done
        if current_step != step_name {
            update_and_send_disko_status(&mut state, sender, &current_step, DiskoStepStatus::Done);
        }
        update_and_send_disko_status(&mut state, sender, &step_name, DiskoStepStatus::InProgress);
        return step_name;
    }

    current_step
}

pub async fn copy_config(work_dir: &str, disk: &str) -> error_stack::Result<(), InstallError> {
    let post = if disk.starts_with("/dev/nvme") {
        // nvme0n1
        // ├─nvme0n1p1
        // ├─nvme0n1p2
        "p3"
    } else {
        // sda
        // └─sda1
        "3"
    };

    let disk = format!("{}{}", disk, post).to_owned();
    let mkdir_args = vec!["mkdir", "-p", "/mnt/data/config"];
    let mount_args = vec!["mount", &disk, "/mnt/data/config"];
    let rsync_args = vec!["rsync", "-av", "--delete", work_dir, "/mnt/data/config"];
    let chown_args = vec!["chown", "-R", "1000:100", "/mnt/data/config"];

    info!("──────────────────────────────────────────────────────────────");
    info!("🚀 Copying system config to system");
    info!("🔧 Running the following commands:");
    info!("     sudo {}", mkdir_args.join(" "));
    info!("     sudo {}", mount_args.join(" "));
    info!("     sudo {}", rsync_args.join(" "));
    info!("     sudo {}", chown_args.join(" "));
    info!("──────────────────────────────────────────────────────────────");

    let commands = vec![mkdir_args, mount_args, rsync_args, chown_args];

    for command in commands {
        let result = exec_simple_command("sudo", command.as_slice())
            .await
            .change_context(InstallError::CopyConfigError)?;

        debug!("{:?}", result);
    }

    Ok(())
}

pub fn update_and_send_disko_status(
    state: &mut EngineState,
    sender: &broadcast::Sender<InstallServerEvent>,
    step_name: &DiskoInstallStepName,
    status: DiskoStepStatus,
) {
    if let Some(step) = state
        .disko_install_steps
        .iter_mut()
        .find(|s| s.name == *step_name)
    {
        step.status = status;
        let _ = sender.send(InstallServerEvent::InstallStepUpdate(step.clone()));
    }
}
