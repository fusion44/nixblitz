use nixblitz_core::*;
use nixblitz_core::{InstallStep, StepStatus};
use nixblitz_system::installer::{
    get_disk_info, get_process_list, get_system_info, perform_system_check,
};
use nixblitz_system::project::Project;
use std::env;
use std::process::{Stdio, exit};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::{Mutex, broadcast};
use tokio::time::sleep;

/// The state of the installation process.
pub struct InstallEngine {
    /// The single source of truth for the state of the installation.
    pub state: InstallState,

    /// The broadcast channel for sending events to clients.
    pub event_sender: broadcast::Sender<ServerEvent>,

    /// The project to operate on
    pub work_dir: String,

    /// The disk to install on
    pub selected_disk: String,
}

pub type SharedInstallEngine = Arc<Mutex<InstallEngine>>;

impl InstallEngine {
    /// Creates a new InstallEngine instance.
    pub fn new() -> Self {
        let (tx, _rx) = broadcast::channel(100);
        let work_dir = env::var(NIXBLITZ_WORK_DIR_ENV);
        let work_dir = match work_dir {
            Ok(v) => v,
            Err(_) => {
                tracing::error!(
                    "Error getting work_dir. Is {} set? Exiting...",
                    NIXBLITZ_WORK_DIR_ENV
                );
                exit(1);
            }
        };

        Self {
            state: InstallState::Idle,
            event_sender: tx,
            work_dir,
            selected_disk: "".to_string(),
        }
    }

    /// The central command processor. It takes a command from a client,
    /// modifies the state, and broadcasts events.
    pub async fn handle_command(&mut self, command: ClientCommand) {
        match command {
            // These commands are in the protocol and must be done after the other
            ClientCommand::PerformSystemCheck => self.perform_system_check().await,
            ClientCommand::UpdateConfig => self.update_config().await,
            ClientCommand::UpdateConfigFinished => self.update_config_finished().await,
            ClientCommand::InstallDiskSelected(disk) => self.install_disk_selected(disk).await,
            ClientCommand::StartInstallation => self.start_installation().await,

            // These commands are out of protocol
            ClientCommand::GetSystemSummary => self.get_system_summary().await,
            ClientCommand::GetProcessList => self.get_process_list().await,
            ClientCommand::DevReset => self.dev_reset().await,
        }
    }

    /// Broadcasts the current state.
    fn broadcast_state(&self) {
        let event = ServerEvent::StateChanged(self.state.clone());
        // We don't care if there are no listeners, so we ignore the error.
        let _ = self.event_sender.send(event);
    }

    // Helper to broadcast events
    async fn broadcast_event(&self, event: ServerEvent) {
        let _ = self.event_sender.send(event);
    }

    async fn perform_system_check(&mut self) {
        self.state = InstallState::PerformingCheck;
        self.broadcast_state();
        // get_system_info() is a blocking call, so we need to wait a bit to let the state be
        // broadcasted
        let _ = sleep(Duration::from_millis(40)).await;

        let summary = get_system_info();
        let check_result = perform_system_check(&summary);

        self.state = InstallState::SystemCheckCompleted(check_result);
        self.broadcast_state();
    }

    async fn get_system_summary(&mut self) {
        let evt = ServerEvent::SystemSummaryUpdated(get_system_info());
        let _ = self.event_sender.send(evt);
    }

    async fn get_process_list(&mut self) {
        let evt = ServerEvent::ProcessListUpdated(get_process_list());
        let _ = self.event_sender.send(evt);
    }

    async fn update_config(&mut self) {
        self.state = InstallState::UpdateConfig;
        self.broadcast_state();
    }

    async fn update_config_finished(&mut self) {
        match get_disk_info() {
            Ok(disks) => self.state = InstallState::SelectInstallDisk(disks),
            Err(e) => self.state = InstallState::InstallFailed(e.to_string()),
        };

        self.broadcast_state();
    }

    async fn install_disk_selected(&mut self, disk: String) {
        match &self.state {
            InstallState::SelectInstallDisk(disk_infos) => {
                let p = Project::load(self.work_dir.clone().into()).unwrap();
                let apps = p.get_enabled_apps();
                let disk_info = disk_infos.iter().find(|d| d.path == disk);
                match disk_info {
                    Some(_) => {
                        self.selected_disk = disk.clone();
                        self.state =
                            InstallState::PreInstallConfirm(PreInstallConfirmData { apps, disk });
                        self.broadcast_state();
                    }
                    None => {
                        self.state = InstallState::SelectDiskError("Disk Not Found".to_owned());
                    }
                }
            }
            _ => {
                self.state = InstallState::SelectDiskError("Selected Disk Not found".to_owned());
            }
        };

        self.broadcast_state();
    }

    async fn start_installation(&mut self) {
        let steps = vec![
            create_step("deps", StepStatus::Waiting),
            create_step("build", StepStatus::Waiting),
            create_step("disk", StepStatus::Waiting),
            create_step("mount", StepStatus::Waiting),
            create_step("copy", StepStatus::Waiting),
            create_step("bootloader", StepStatus::Waiting),
        ];

        self.state = InstallState::Installing(steps);
        self.broadcast_state();

        let event_sender = self.event_sender.clone();

        tokio::spawn(fake_install_process(event_sender));
    }

    // async fn start_installation(&mut self) {
    //     // // 1. Check if we are in the correct state to start
    //     // if let InstallState::SystemCheckCompleted(ref result) = self.state {
    //     //     if !result.is_compatible {
    //     //         self.broadcast_event(ServerEvent::Error(
    //     //             "Cannot install on an incompatible system.".into(),
    //     //         ))
    //     //         .await;
    //     //         return;
    //     //     }
    //     // } else {
    //     //     self.broadcast_event(ServerEvent::Error(
    //     //         "System check must be performed before installation.".into(),
    //     //     ))
    //     //     .await;
    //     //     return;
    //     // }
    //
    //     // 2. Define our installation steps
    //     let steps = vec![
    //         InstallStep {
    //             id: "deps".into(),
    //             name: "Fetching Dependencies".into(),
    //             status: StepStatus::Waiting,
    //         },
    //         InstallStep {
    //             id: "build".into(),
    //             name: "Building NixOS System".into(),
    //             status: StepStatus::Waiting,
    //         },
    //         InstallStep {
    //             id: "disk".into(),
    //             name: "Partitioning & Formatting Disk".into(),
    //             status: StepStatus::Waiting,
    //         },
    //         InstallStep {
    //             id: "mount".into(),
    //             name: "Mounting Filesystems".into(),
    //             status: StepStatus::Waiting,
    //         },
    //         InstallStep {
    //             id: "copy".into(),
    //             name: "Copying System to Disk".into(),
    //             status: StepStatus::Waiting,
    //         },
    //         InstallStep {
    //             id: "bootloader".into(),
    //             name: "Installing Bootloader".into(),
    //             status: StepStatus::Waiting,
    //         },
    //     ];
    //
    //     // 3. Set the initial state to Installing and broadcast it
    //     self.state = InstallState::Installing(steps.clone());
    //     self.broadcast_state();
    //
    //     // 4. Prepare for the background task
    //     let event_sender = self.event_sender.clone();
    //
    //     // TODO: Get these from the client command or stored state
    //     let work_dir = self.work_dir.clone();
    //     let nixos_config_name = "nixblitzvm".to_string();
    //     let disk = self.selected_disk.clone();
    //
    //     // 5. Spawn the long-running installation task
    //     tokio::spawn(async move {
    //         let mut child = match Command::new("sudo")
    //             .arg("disko-install")
    //             .arg("--flake")
    //             .arg(format!("{}/src#{}", &work_dir, &nixos_config_name))
    //             .arg("--disk")
    //             .arg("main")
    //             .arg(&disk)
    //             .stdout(Stdio::piped())
    //             .stderr(Stdio::piped())
    //             .spawn()
    //         {
    //             Ok(child) => child,
    //             Err(e) => {
    //                 let _ = event_sender.send(ServerEvent::StateChanged(
    //                     InstallState::InstallFailed(e.to_string()),
    //                 ));
    //                 return;
    //             }
    //         };
    //
    //         let stdout = child.stdout.take().expect("Failed to capture stdout");
    //         let stderr = child.stderr.take().expect("Failed to capture stderr");
    //
    //         let mut reader = BufReader::new(stdout).lines();
    //         let mut err_reader = BufReader::new(stderr).lines();
    //
    //         let mut current_step_id = "";
    //
    //         loop {
    //             tokio::select! {
    //                 // Process lines from stdout
    //                 Ok(Some(line)) = reader.next_line() => {
    //                     let _ = event_sender.send(ServerEvent::InstallLog(line.clone()));
    //
    //                     // --- Landmark Parsing Logic ---
    //                     if line.contains("unpacking 'github:") && current_step_id != "deps" {
    //                         update_step_status(&event_sender, "deps", StepStatus::InProgress).await;
    //                         current_step_id = "deps";
    //                     } else if line.contains("derivations will be built") && current_step_id != "build" {
    //                         update_step_status(&event_sender, "deps", StepStatus::Done).await;
    //                         update_step_status(&event_sender, "build", StepStatus::InProgress).await;
    //                         current_step_id = "build";
    //                     } else if line.contains("sgdisk") && current_step_id != "disk" {
    //                         update_step_status(&event_sender, "build", StepStatus::Done).await;
    //                         update_step_status(&event_sender, "disk", StepStatus::InProgress).await;
    //                         current_step_id = "disk";
    //                     } else if line.contains("mount /dev/disk") && current_step_id != "mount" {
    //                         update_step_status(&event_sender, "disk", StepStatus::Done).await;
    //                         update_step_status(&event_sender, "mount", StepStatus::InProgress).await;
    //                         current_step_id = "mount";
    //                     } else if line.contains("Copying store paths") && current_step_id != "copy" {
    //                         update_step_status(&event_sender, "mount", StepStatus::Done).await;
    //                         update_step_status(&event_sender, "copy", StepStatus::InProgress).await;
    //                         current_step_id = "copy";
    //                     } else if line.contains("installing the boot loader") && current_step_id != "bootloader" {
    //                         update_step_status(&event_sender, "copy", StepStatus::Done).await;
    //                         update_step_status(&event_sender, "bootloader", StepStatus::InProgress).await;
    //                         current_step_id = "bootloader";
    //                     }
    //                 },
    //                 // Process lines from stderr
    //                 Ok(Some(line)) = err_reader.next_line() => {
    //                     // Send stderr lines as logs too, maybe with a prefix
    //                     let _ = event_sender.send(ServerEvent::InstallLog(format!("[STDERR] {}", line)));
    //                     // TODO: Add error pattern detection here
    //                 },
    //                 else => break,
    //             }
    //         }
    //
    //         // 6. Check the final status of the command
    //         match child.wait().await {
    //             Ok(status) if status.success() => {
    //                 update_step_status(&event_sender, "bootloader", StepStatus::Done).await;
    //                 let _ = event_sender
    //                     .send(ServerEvent::StateChanged(InstallState::InstallSucceeded));
    //             }
    //             Ok(status) => {
    //                 let err_msg = format!("Installation failed with exit code: {}", status);
    //                 update_step_status(
    //                     &event_sender,
    //                     current_step_id,
    //                     StepStatus::Failed(err_msg.clone()),
    //                 )
    //                 .await;
    //                 let _ = event_sender.send(ServerEvent::StateChanged(
    //                     InstallState::InstallFailed(err_msg),
    //                 ));
    //             }
    //             Err(e) => {
    //                 let err_msg = format!("Failed to wait for installer command: {}", e);
    //                 update_step_status(
    //                     &event_sender,
    //                     current_step_id,
    //                     StepStatus::Failed(err_msg.clone()),
    //                 )
    //                 .await;
    //                 let _ = event_sender.send(ServerEvent::StateChanged(
    //                     InstallState::InstallFailed(err_msg),
    //                 ));
    //             }
    //         }
    //     });
    // }

    async fn dev_reset(&mut self) {
        self.state = InstallState::Idle;
        self.broadcast_state();
    }
}

/// A helper function to create an `InstallStep` object, reducing boilerplate.
fn create_step(id: &str, status: StepStatus) -> InstallStep {
    let name = match id {
        "deps" => "Fetching Dependencies",
        "build" => "Building NixOS System",
        "disk" => "Partitioning & Formatting Disk",
        "mount" => "Mounting Filesystems",
        "copy" => "Copying System to Disk",
        "bootloader" => "Installing Bootloader",
        _ => "Unknown Step",
    }
    .to_string();

    InstallStep {
        id: id.to_string(),
        name,
        status,
    }
}

/// This function simulates the entire installation process, sending events
/// to the provided sender to update clients.
async fn fake_install_process(sender: broadcast::Sender<ServerEvent>) {
    // --- Configuration for the simulation ---
    // Set this to `true` to test your UI's failure state.
    let simulate_failure = false;
    // The ID of the step where the failure should occur.
    let failure_step_id = "disk";
    // -----------------------------------------

    let step_ids = ["deps", "build", "disk", "mount", "copy", "bootloader"];

    for step_id in step_ids {
        // 1. Mark the current step as InProgress
        let _ = sender.send(ServerEvent::InstallStepUpdate(create_step(
            step_id,
            StepStatus::InProgress,
        )));
        let _ = sender.send(ServerEvent::InstallLog(format!(
            "Starting step: {}",
            create_step(step_id, StepStatus::Waiting).name
        )));

        // 2. Simulate work with a sleep
        let sleep_duration_secs = match step_id {
            "build" => 4, // Make build and copy take longer
            "copy" => 3,
            _ => 1,
        };
        tokio::time::sleep(Duration::from_secs(sleep_duration_secs)).await;

        let _ = sender.send(ServerEvent::InstallLog("...".to_string()));

        // 3. Check if we should simulate a failure at this step
        if simulate_failure && step_id == failure_step_id {
            let reason = "Failed to format partition (simulated error).".to_string();
            let _ = sender.send(ServerEvent::InstallStepUpdate(create_step(
                step_id,
                StepStatus::Failed(reason.clone()),
            )));
            let _ = sender.send(ServerEvent::StateChanged(InstallState::InstallFailed(
                reason,
            )));
            return; // End the simulation here
        }

        // 4. Mark the current step as Done
        let _ = sender.send(ServerEvent::InstallStepUpdate(create_step(
            step_id,
            StepStatus::Done,
        )));
    }

    // 5. If we finished the loop without failing, the installation was a success!
    tracing::debug!("Fake installation process completed successfully.");
    let _ = sender.send(ServerEvent::StateChanged(InstallState::InstallSucceeded));
}

// Helper function to reduce boilerplate
async fn update_step_status(sender: &broadcast::Sender<ServerEvent>, id: &str, status: StepStatus) {
    // This mapping could be more robust, but it's fine for now
    let name = match id {
        "deps" => "Fetching Dependencies",
        "build" => "Building NixOS System",
        "disk" => "Partitioning & Formatting Disk",
        "mount" => "Mounting Filesystems",
        "copy" => "Copying System to Disk",
        "bootloader" => "Installing Bootloader",
        _ => "Unknown Step",
    };

    let step_update = InstallStep {
        id: id.to_string(),
        name: name.to_string(),
        status,
    };
    let _ = sender.send(ServerEvent::InstallStepUpdate(step_update));
}
