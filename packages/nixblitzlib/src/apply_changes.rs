use core::fmt;
use error_stack::{Report, Result};
use log::{debug, error, info};
use std::process::{ExitStatus, Stdio};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::task;

use crate::errors::CommandError;

#[derive(Debug, Clone)]
pub enum ProcessOutput {
    Stdout(String),
    Stderr(String),
    Completed(ExitStatus),
    Error(String),
}

impl fmt::Display for ProcessOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessOutput::Stdout(output) => write!(f, "Stdout: {}", output),
            ProcessOutput::Stderr(output) => write!(f, "Stderr: {}", output),
            ProcessOutput::Completed(status) => write!(f, "Completed with status: {}", status),
            ProcessOutput::Error(err) => write!(f, "Error: {}", err),
        }
    }
}

/// Runs `nixos-rebuild switch` asynchronously using Tokio and streams its output.
///
/// Returns a Tokio MPSC `Receiver` channel that will emit `ProcessOutput` variants.
/// Returns a `CommandStartError` if the process cannot be started.
pub async fn run_nixos_rebuild_switch_async(
    work_dir: String,
) -> Result<UnboundedReceiver<ProcessOutput>, CommandError> {
    let (tx, rx) = unbounded_channel();

    let command_str = "nixos-rebuild";
    let system = format!("{}/src#nixblitzx86", work_dir.clone());
    // TODO: add a debug mode. Possible flags:
    // "--show-trace",
    // "--print-build-logs",
    // "--verbose",
    let args = ["switch", "--flake", system.as_str(), "--impure"];

    let mut cmd = Command::new(command_str);
    cmd.args(args).stdout(Stdio::piped()).stderr(Stdio::piped());

    let mut child = cmd
        .spawn()
        .map_err(|e| CommandError::SpawnFailed(e.to_string()))?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| CommandError::PipeError("Could not capture stdout".to_string()))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| CommandError::PipeError("Could not capture stderr".to_string()))?;

    task::spawn(manage_process(child, stdout, stderr, tx));

    Ok(rx)
}

// Background task function to handle I/O and process waiting
async fn manage_process(
    mut child: Child,
    stdout: tokio::process::ChildStdout,
    stderr: tokio::process::ChildStderr,
    tx: UnboundedSender<ProcessOutput>,
) -> Result<(), CommandError> {
    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();

    let mut stdout_done = false;
    let mut stderr_done = false;

    loop {
        tokio::select! {
            biased;
            result = stdout_reader.next_line(), if !stdout_done => {
                match result {
                    Ok(Some(line)) => {
                        if tx.send(ProcessOutput::Stdout(line)).is_err() {
                             error!("Output channel receiver dropped (stdout). Stopping task.");
                             break;
                        }
                    },
                    Ok(None) => {
                        stdout_done = true;
                        debug!("Stdout stream finished.");
                    },
                    Err(e) => {
                        let err_msg = format!("Error reading stdout: {}", e);
                        error!("{}", &err_msg);
                        if tx.send(ProcessOutput::Error(err_msg)).is_err() {
                            error!("Output channel receiver dropped (stdout error). Stopping task.");
                            break;
                        }
                        // Assume stream is unusable after error
                        stdout_done = true;
                    }
                }
            },

            result = stderr_reader.next_line(), if !stderr_done => {
                 match result {
                    Ok(Some(line)) => {
                        println!("line: {}", line);
                        if tx.send(ProcessOutput::Stderr(line)).is_err() {
                            error!("Output channel receiver dropped (stderr). Stopping task.");
                            break;
                        }
                    },
                    Ok(None) => {
                        stderr_done = true;
                        debug!("Stderr stream finished.");
                    },
                    Err(e) => {
                        let err_msg = format!("Error reading stderr: {}", e);
                        error!("{}", &err_msg);
                        if tx.send(ProcessOutput::Error(err_msg)).is_err() {
                           error!("Output channel receiver dropped (stderr error). Stopping task.");
                           break;
                        }
                        stderr_done = true;
                    }
                }
            },

            else => {
                if stdout_done && stderr_done {
                    debug!("Both stdout and stderr streams finished. Breaking select loop.");
                    break;
                }
            }
        }
    }

    info!("Waiting for process exit...");
    match child.wait().await {
        Ok(status) => {
            info!("Process exited with status: {}", status);
            if tx.send(ProcessOutput::Completed(status)).is_err() {
                error!("Output channel receiver dropped before sending completion status.");
            }
        }
        Err(e) => {
            let err_msg = format!("Error waiting for process exit: {}", e);
            error!("{}", err_msg.clone());
            if tx.send(ProcessOutput::Error(err_msg.clone())).is_err() {
                error!("Output channel receiver dropped before sending wait error.");
            }
            return Err(Report::new(CommandError::PipeError(err_msg)));
        }
    }

    info!("Manage process task finished successfully.");
    Ok(())
}
