use log::{debug, error, info};
use nixblitz_core::nix_base::NixBaseConfigOption;
use nixblitz_core::option_data::{OptionDataChangeNotification, ToOptionId};
use nixblitz_core::text_edit_data::TextOptionChangeData;
use nixblitz_core::*;
use nixblitz_core::{DiskoInstallStep, DiskoStepStatus};
use nixblitz_system::installer::{
    get_disk_info, get_process_list, get_system_info, perform_system_check,
};
use nixblitz_system::project::Project;
use nixblitz_system::utils::{check_system_dependencies, commit_config};
use std::env;
use std::process::exit;
use std::sync::Arc;
use std::time::Duration;
use strum::VariantArray;
use tokio::sync::{Mutex, broadcast};
use tokio::time::sleep;

use crate::core::{copy_config, install_system, update_and_send_disko_status};

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

    /// Whether to use the demo config. This will fake the installation process.
    pub is_demo: bool,
}

pub type SharedInstallState = Arc<Mutex<EngineState>>;

impl InstallEngine {
    /// Creates a new InstallEngine instance.
    pub fn new(is_demo: bool) -> Self {
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
            is_demo,
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
        let state_arc = self.state.clone();
        let sender_clone = self.event_sender.clone();

        if self.is_demo {
            let state_clone = self.state.clone();
            let sender_clone = self.event_sender.clone();

            {
                let mut state = state_clone.lock().await;
                state.install_state = InstallState::Installing(state.disko_install_steps.clone());
                self.broadcast_state(&state.install_state);
            }

            tokio::spawn(async move { fake_install_process(state_clone, sender_clone).await });
        } else {
            let (work_dir, nixos_config_name, disk) = {
                let mut state = state_arc.lock().await;

                if !matches!(state.install_state, InstallState::PreInstallConfirm(_)) {
                    let _ = self.event_sender.send(ServerEvent::Error(
                        "Not in correct state to start installation.".to_string(),
                    ));
                    return;
                }

                state.install_state = InstallState::Installing(state.disko_install_steps.clone());
                self.broadcast_state(&state.install_state);

                // TODO: The nixos_config_name should not be hardcoded
                (
                    self.work_dir.clone(),
                    "nixblitzx86vm".to_string(),
                    state.selected_disk.clone(),
                )
            };

            tokio::spawn(async move {
                real_install_process(state_arc, sender_clone, work_dir, nixos_config_name, disk)
                    .await;
            });
        }
    }

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

async fn real_install_process(
    state_arc: SharedInstallState,
    sender: broadcast::Sender<ServerEvent>,
    work_dir: String,
    // The NixOS configuration name from your flake, e.g., "nixblitzvm"
    nixos_config_name: String,
    // The target disk, e.g., "/dev/sda" or "/dev/vda"
    disk: String,
) {
    info!("Starting real_install_process");

    let res = check_system_dependencies(&[
        "chown",
        "disko-install",
        "git",
        "mkdir",
        "mount",
        "rsync",
        "sudo",
    ]);
    if let Err(missing) = res {
        let error_msg = format!("Missing system dependencies: {}", missing.join(", "));
        error!("{}", error_msg);
        let mut state = state_arc.lock().await;
        state.install_state = InstallState::InstallFailed(error_msg);
        let _ = sender.send(ServerEvent::StateChanged(state.install_state.clone()));
        return;
    }

    let res = install_system(
        state_arc.clone(),
        sender.clone(),
        &work_dir,
        &nixos_config_name,
        &disk,
    )
    .await;

    if res.is_err() {
        // The function does notify the client about the error, so we don't need to do it here.
        error!("{}", res.unwrap_err());
        return;
    }

    {
        let mut state = state_arc.lock().await;
        update_and_send_disko_status(
            &mut state,
            &sender,
            &DiskoInstallStepName::PostInstall,
            DiskoStepStatus::InProgress,
        );
        add_and_send_install_log(
            &mut state,
            &sender,
            "Git committing system config...".to_string(),
        );
    }

    {
        let mut p = Project::load(work_dir.clone().into()).unwrap();
        let change_notification =
            OptionDataChangeNotification::TextEdit(TextOptionChangeData::new(
                NixBaseConfigOption::DiskoDevice.to_option_id(),
                disk.clone(),
            ));
        p.on_option_changed(change_notification).unwrap();
    }

    let res = commit_config(work_dir.as_str(), "init system").await;
    match res {
        Ok(_) => {} // Do nothing
        Err(e) => {
            let err_str = format!("{:?}", e);
            error!("{}", err_str.clone());
            let mut state = state_arc.lock().await;
            update_and_send_disko_status(
                &mut state,
                &sender,
                &DiskoInstallStepName::PostInstall,
                DiskoStepStatus::Failed(err_str.clone()),
            );
            add_and_send_install_log(&mut state, &sender, err_str);
            return;
        }
    };

    {
        let mut state = state_arc.lock().await;
        add_and_send_install_log(&mut state, &sender, "Copying system config...".to_string());
    }

    let res = copy_config(&work_dir, &disk).await;
    match res {
        Ok(_) => {} // Do nothing
        Err(e) => {
            let err_str = format!("{:?}", e);
            error!("{}", err_str.clone());
            let mut state = state_arc.lock().await;
            update_and_send_disko_status(
                &mut state,
                &sender,
                &DiskoInstallStepName::PostInstall,
                DiskoStepStatus::Failed(e.to_string()),
            );
            add_and_send_install_log(&mut state, &sender, err_str);
            return;
        }
    };

    {
        let mut state = state_arc.lock().await;
        update_and_send_disko_status(
            &mut state,
            &sender,
            &DiskoInstallStepName::PostInstall,
            DiskoStepStatus::Done,
        );
    }

    {
        let mut state_guard = state_arc.lock().await;
        let final_steps = state_guard.disko_install_steps.clone();
        state_guard.install_state = InstallState::InstallSucceeded(final_steps);
        debug!("Installation process completed successfully.");
        let final_event = ServerEvent::StateChanged(state_guard.install_state.clone());
        let _ = sender.send(final_event);
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
            update_and_send_disko_status(
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
            update_and_send_disko_status(
                &mut state_guard,
                &sender,
                step_name,
                DiskoStepStatus::Failed(reason.clone()),
            );

            return;
        }

        let mut state_guard = state_arc.lock().await;
        update_and_send_disko_status(&mut state_guard, &sender, step_name, DiskoStepStatus::Done);
    }

    {
        let mut state_guard = state_arc.lock().await;
        update_and_send_disko_status(
            &mut state_guard,
            &sender,
            &DiskoInstallStepName::PostInstall,
            DiskoStepStatus::InProgress,
        );
    }

    tokio::time::sleep(Duration::from_secs(1)).await;

    {
        let mut state_guard = state_arc.lock().await;
        update_and_send_disko_status(
            &mut state_guard,
            &sender,
            &DiskoInstallStepName::PostInstall,
            DiskoStepStatus::Done,
        );
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
        DiskoInstallStep {
            name: DiskoInstallStepName::PostInstall,
            status: DiskoStepStatus::Waiting,
        },
    ]
}
