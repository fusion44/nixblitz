use crate::engine::SharedSystemState;
use error_stack::Report;
use log::info;
use nixblitz_core::{SystemError, SystemServerEvent, SystemState};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::broadcast;

pub(crate) async fn switch_config(
    state_arc: SharedSystemState,
    sender: broadcast::Sender<SystemServerEvent>,
    work_dir: &String,
) -> error_stack::Result<(), SystemError> {
    let args = &["nixblitz", "apply", "--work-dir", work_dir];
    let cmd_str = format!("doas {}", args.join(" "));
    info!("──────────────────────────────────────────────────────────────");
    info!("🚀 Applying system config in '{}'", work_dir);
    info!("🔧 Running command via doas:");
    info!("   doas {}", args.join(" "));
    info!("──────────────────────────────────────────────────────────────");

    let mut child = match Command::new("doas")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(e) => {
            // TODO: Notify the client about the error

            return Err(Report::new(SystemError::CommandError(
                cmd_str,
                e.to_string(),
            )));
        }
    };

    let stdout = child.stdout.take().expect("Failed to capture stdout");
    let stderr = child.stderr.take().expect("Failed to capture stderr");

    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();

    loop {
        tokio::select! {
            result = stdout_reader.next_line() => {
                match result {
                    Ok(Some(line)) => {
                        info!("[STDOUT]: {}", line);
                        let _ = sender.send(SystemServerEvent::UpdateLog(line.clone()));
                    },
                    _ => break,
                }
            },
            result = stderr_reader.next_line() => {
                match result {
                    Ok(Some(line)) => {
                        info!("[STDERR]: {}", line);
                        let _ = sender.send(SystemServerEvent::UpdateLog(format!("[STDERR] {}", line)));
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
            state.state = SystemState::Idle;
            let _ = sender.send(SystemServerEvent::StateChanged(state.state.clone()));
            return Err(Report::new(SystemError::CommandError(
                cmd_str,
                e.to_string(),
            )));
        }
    };

    // TODO: handle cases where the application succeeds, but a service fails to start
    if status.success() {
        info!("Switch to new config succeeded.");
        let mut state = state_arc.lock().await;
        state.state = SystemState::Idle;
    } else {
        let msg = format!("Switch to new config failed with exit code: {}", status);
        let mut state = state_arc.lock().await;
        state.state = SystemState::Idle;
        // TODO: Notify the client about the error
        return Err(Report::new(SystemError::CommandError(cmd_str, msg)));
    }

    Ok(())
}
