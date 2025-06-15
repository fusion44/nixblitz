use log::{debug, error, info};
use nixblitz_core::*;
use nixblitz_core::{DiskoInstallStep, DiskoStepStatus};
use nixblitz_system::installer::{
    get_disk_info, get_process_list, get_system_info, perform_system_check,
};
use nixblitz_system::project::Project;
use std::env;
use std::process::exit;
use std::sync::Arc;
use std::time::Duration;
use strum::VariantArray;
use tokio::sync::{Mutex, broadcast};
use tokio::time::sleep;

pub struct EngineState {
    /// The actual state of the installation process.
    pub install_state: InstallState,

    /// The disk to install on
    pub selected_disk: String,

    /// The steps to perform during the installation
    pub disko_install_steps: Vec<DiskoInstallStep>,

    /// The current log entries
    pub logs: Vec<String>,
}

pub struct InstallEngine {
    /// The central state of the installation process.
    pub state: SharedInstallState,

    /// The broadcast channel for sending events to clients.
    pub event_sender: broadcast::Sender<ServerEvent>,

    /// The project to operate on
    pub work_dir: String,
}

pub type SharedInstallState = Arc<Mutex<EngineState>>;

impl InstallEngine {
    /// Creates a new InstallEngine instance.
    pub fn new() -> Self {
        let (tx, _rx) = broadcast::channel(100);
        let work_dir = env::var(NIXBLITZ_WORK_DIR_ENV);
        let work_dir = match work_dir {
            Ok(v) => v,
            Err(_) => {
                error!(
                    "Error getting work_dir. Is {} set? Exiting...",
                    NIXBLITZ_WORK_DIR_ENV
                );
                exit(1);
            }
        };

        let initial_state = EngineState {
            install_state: InstallState::Idle,
            selected_disk: "".to_string(),
            disko_install_steps: initialize_disko_install_steps(),
            logs: vec![],
        };

        Self {
            state: Arc::new(Mutex::new(initial_state)),
            event_sender: tx,
            work_dir,
        }
    }

    /// The central command processor. It takes a command from a client,
    /// modifies the state, and broadcasts events.
    pub async fn handle_command(&self, command: ClientCommand) {
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
    fn broadcast_state(&self, state: &InstallState) {
        let event = ServerEvent::StateChanged(state.clone());
        // We don't care if there are no listeners, so we ignore the error.
        let _ = self.event_sender.send(event);
    }

    async fn perform_system_check(&self) {
        let mut state = self.state.lock().await;
        state.install_state = InstallState::PerformingCheck;
        self.broadcast_state(&state.install_state);
        // get_system_info() is a blocking call, so we need to wait a bit to
        // let the state be broadcasted
        drop(state);

        let _ = sleep(Duration::from_millis(40)).await;

        // TODO: This blocks the thread, so we need to do it in a separate task
        let summary = get_system_info();
        let check_result = perform_system_check(&summary);

        let mut state = self.state.lock().await;
        state.install_state = InstallState::SystemCheckCompleted(check_result);
        self.broadcast_state(&state.install_state);
        info!("System check completed.");
    }

    async fn get_system_summary(&self) {
        let evt = ServerEvent::SystemSummaryUpdated(get_system_info());
        let _ = self.event_sender.send(evt);
    }

    async fn get_process_list(&self) {
        let evt = ServerEvent::ProcessListUpdated(get_process_list());
        let _ = self.event_sender.send(evt);
    }

    async fn update_config(&self) {
        let mut state = self.state.lock().await;
        state.install_state = InstallState::UpdateConfig;
        self.broadcast_state(&state.install_state);
    }

    async fn update_config_finished(&self) {
        let mut state = self.state.lock().await;
        match get_disk_info() {
            Ok(disks) => state.install_state = InstallState::SelectInstallDisk(disks),
            Err(e) => {
                error!("Failed to get disk info: {}", e);
                state.install_state = InstallState::InstallFailed(e.to_string())
            }
        };

        self.broadcast_state(&state.install_state);
    }

    async fn install_disk_selected(&self, disk: String) {
        let mut state = self.state.lock().await;

        match &state.install_state {
            InstallState::SelectInstallDisk(disk_infos) => {
                let p = Project::load(self.work_dir.clone().into()).unwrap();
                let apps = p.get_enabled_apps();
                let disk_info = disk_infos.iter().find(|d| d.path == disk);
                match disk_info {
                    Some(_) => {
                        state.selected_disk = disk.clone();
                        state.install_state =
                            InstallState::PreInstallConfirm(PreInstallConfirmData { apps, disk });
                    }
                    None => {
                        state.install_state =
                            InstallState::SelectDiskError("Disk Not Found".to_owned());
                    }
                }
            }
            _ => {
                state.install_state =
                    InstallState::SelectDiskError("Selected Disk Not found".to_owned());
            }
        };

        self.broadcast_state(&state.install_state);
    }

    async fn start_installation(&self) {
        let state_clone = self.state.clone();
        let sender_clone = self.event_sender.clone();

        {
            let mut state = state_clone.lock().await;
            state.install_state = InstallState::Installing(state.disko_install_steps.clone());
            self.broadcast_state(&state.install_state);
        }

        tokio::spawn(async move { fake_install_process(state_clone, sender_clone).await });
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

    async fn dev_reset(&self) {
        let mut state = self.state.lock().await;
        state.install_state = InstallState::Idle;
        state.disko_install_steps = initialize_disko_install_steps();
        state.logs = vec![];
        self.broadcast_state(&state.install_state);
    }
}

fn add_and_send_install_log(
    state: &mut EngineState,
    sender: &broadcast::Sender<ServerEvent>,
    log: String,
) {
    state.logs.push(log.clone());
    let _ = sender.send(ServerEvent::InstallLog(log));
}

fn update_and_send_status(
    state: &mut EngineState,
    sender: &broadcast::Sender<ServerEvent>,
    step_name: &DiskoInstallStepName,
    status: DiskoStepStatus,
) {
    if let Some(step) = state
        .disko_install_steps
        .iter_mut()
        .find(|s| s.name == *step_name)
    {
        step.status = status;
        let _ = sender.send(ServerEvent::InstallStepUpdate(step.clone()));
    }
}

/// This function simulates the entire installation process, sending events
/// to the provided sender to update clients.
async fn fake_install_process(
    state_arc: SharedInstallState,
    sender: broadcast::Sender<ServerEvent>,
) {
    let simulate_failure = false;
    let failure_step_id = DiskoInstallStepName::Disk;

    for step_name in DiskoInstallStepName::VARIANTS {
        {
            let mut state_guard = state_arc.lock().await;
            update_and_send_status(
                &mut state_guard,
                &sender,
                step_name,
                DiskoStepStatus::InProgress,
            );
            add_and_send_install_log(
                &mut state_guard,
                &sender,
                format!("Starting step: {}", step_name),
            );
        }

        let sleep_duration_secs = match step_name {
            DiskoInstallStepName::Build => 4, // Make build and copy take longer
            DiskoInstallStepName::Copy => 3,
            _ => 1,
        };
        tokio::time::sleep(Duration::from_secs(sleep_duration_secs)).await;

        {
            let mut state_guard = state_arc.lock().await;
            add_and_send_install_log(&mut state_guard, &sender, "...".to_string());
        }

        if simulate_failure && *step_name == failure_step_id {
            let reason = "Failed to format partition (simulated error).".to_string();

            let mut state_guard = state_arc.lock().await;
            update_and_send_status(
                &mut state_guard,
                &sender,
                step_name,
                DiskoStepStatus::Failed(reason.clone()),
            );

            return;
        }

        let mut state_guard = state_arc.lock().await;
        update_and_send_status(&mut state_guard, &sender, step_name, DiskoStepStatus::Done);
    }

    {
        let mut state_guard = state_arc.lock().await;
        let final_steps = state_guard.disko_install_steps.clone();
        state_guard.install_state = InstallState::InstallSucceeded(final_steps);
        debug!("Fake installation process completed successfully.");
        let final_event = ServerEvent::StateChanged(state_guard.install_state.clone());
        let _ = sender.send(final_event);
    }
}

fn initialize_disko_install_steps() -> Vec<DiskoInstallStep> {
    vec![
        DiskoInstallStep {
            name: DiskoInstallStepName::Deps,
            status: DiskoStepStatus::Waiting,
        },
        DiskoInstallStep {
            name: DiskoInstallStepName::Build,
            status: DiskoStepStatus::Waiting,
        },
        DiskoInstallStep {
            name: DiskoInstallStepName::Disk,
            status: DiskoStepStatus::Waiting,
        },
        DiskoInstallStep {
            name: DiskoInstallStepName::Mount,
            status: DiskoStepStatus::Waiting,
        },
        DiskoInstallStep {
            name: DiskoInstallStepName::Copy,
            status: DiskoStepStatus::Waiting,
        },
        DiskoInstallStep {
            name: DiskoInstallStepName::Bootloader,
            status: DiskoStepStatus::Waiting,
        },
    ]
}
