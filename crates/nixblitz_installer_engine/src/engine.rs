use nixblitz_core::*;
use nixblitz_system::installer::{
    get_disk_info, get_process_list, get_system_info, perform_system_check,
};
use nixblitz_system::project::Project;
use std::env;
use std::process::exit;
use std::sync::Arc;
use std::time::Duration;
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
        }
    }

    /// The central command processor. It takes a command from a client,
    /// modifies the state, and broadcasts events.
    pub async fn handle_command(&mut self, command: ClientCommand) {
        match command {
            ClientCommand::PerformSystemCheck => self.perform_system_check().await,
            ClientCommand::GetSystemSummary => self.get_system_summary().await,
            ClientCommand::GetProcessList => self.get_process_list().await,
            ClientCommand::DevReset => self.dev_reset().await,
            ClientCommand::InstallDiskSelected(disk) => self.install_disk_selected(disk).await,
            ClientCommand::StartInstallation => self.start_installation().await,
            ClientCommand::UpdateConfig => self.update_config().await,
            ClientCommand::UpdateConfigFinished => self.update_config_finished().await,
        }
    }

    /// Broadcasts the current state.
    fn broadcast_state(&self) {
        let event = ServerEvent::StateChanged(self.state.clone());
        // We don't care if there are no listeners, so we ignore the error.
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
        self.state = InstallState::Installing("Starting the install process".into());
        self.broadcast_state();
    }

    async fn dev_reset(&mut self) {
        self.state = InstallState::Idle;
        self.broadcast_state();
    }
}
