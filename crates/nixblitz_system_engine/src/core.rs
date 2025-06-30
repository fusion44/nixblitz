use std::path::PathBuf;

use crate::engine::SharedSystemState;
use error_stack::{Report, Result, ResultExt};
use log::info;
use nixblitz_core::{SystemError, SystemServerEvent, SystemState};
use nixblitz_system::{
    apply_changes::{ProcessOutput, run_nixos_rebuild_switch_async},
    project::Project,
};
use tokio::sync::{broadcast, mpsc::UnboundedReceiver};

pub(crate) async fn switch_config(
    state_arc: SharedSystemState,
    sender: broadcast::Sender<SystemServerEvent>,
    work_dir: &String,
) -> error_stack::Result<(), SystemError> {
    info!("──────────────────────────────────────────────────────────────");
    info!("🚀 Applying system config in '{}'", work_dir);
    info!("──────────────────────────────────────────────────────────────");
    match run_nixos_rebuild_switch_async(
        work_dir.to_string(),
        &nixblitz_core::SystemPlatform::X86_64Vm,
    )
    .await
    {
        Ok(receiver) => {
            process_output(state_arc.clone(), sender.clone(), receiver, work_dir).await?;
            Ok(())
        }
        Err(report) => Err(report.change_context(SystemError::UpdateError(
            "Failed to apply config.".to_string(),
        ))),
    }
}

async fn process_output(
    state_arc: SharedSystemState,
    sender: broadcast::Sender<SystemServerEvent>,
    mut receiver: UnboundedReceiver<ProcessOutput>,
    work_dir: &String,
) -> Result<(), SystemError> {
    while let Some(output) = receiver.recv().await {
        match output {
            ProcessOutput::Stdout(line) => {
                info!("[STDOUT]: {}", line);
                let _ = sender.send(SystemServerEvent::UpdateLog(line.clone()));
            }
            ProcessOutput::Stderr(line) => {
                info!("[STDERR]: {}", line);
                let _ = sender.send(SystemServerEvent::UpdateLog(format!("[STDERR] {}", line)));
            }
            ProcessOutput::Completed(status) => {
                info!("--- Process finished with status: {} ---", status);
                if !status.success() {
                    Err(Report::new(SystemError::UpdateError(
                        "Warning: Command exited with non-zero status.".to_string(),
                    )))?
                } else {
                    let mut project = Project::load(PathBuf::from(work_dir)).change_context(
                        SystemError::UpdateError("Unable to load project.".to_string()),
                    )?;
                    project
                        .set_changes_applied()
                        .await
                        .change_context(SystemError::UpdateError(
                            "Unable to apply changes to project.".to_string(),
                        ))?
                }

                // TODO: handle cases where the application succeeds, but a service fails to start
                if status.success() {
                    info!("Switch to new config succeeded.");
                    let mut state = state_arc.lock().await;
                    state.state = SystemState::Idle;
                } else {
                    let msg = format!("Switch to new config failed with exit code: {}", status);
                    let mut state = state_arc.lock().await;
                    state.state = SystemState::Idle;
                    let _ = sender.send(SystemServerEvent::Error(msg.clone()));
                    return Err(Report::new(SystemError::UpdateError(msg)));
                }
            }
            ProcessOutput::Error(err_msg) => {
                info!("RUNTIME ERROR: {}", err_msg);
                break;
            }
        }
    }

    Ok(())
}
