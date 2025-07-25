use log::{debug, error, info};
use nixblitz_core::{SystemState, *};
use nixblitz_system::project::Project;
use nixblitz_system::utils::reboot_system;
use std::env;
use std::process::exit;
use std::sync::Arc;
use std::time::Duration;
use strum::VariantArray;
use tokio::sync::{Mutex, broadcast};

use crate::core::switch_config;

pub(crate) struct EngineInternalState {
    /// The actual state of the system.
    pub state: nixblitz_core::SystemState,

    /// The project to operate on
    pub project: Project,

    /// The current log entries
    pub logs: Vec<String>,
}

pub struct SystemEngine {
    /// The central state of the system process.
    pub state: SharedSystemState,

    /// The broadcast channel for sending events to clients.
    pub event_sender: broadcast::Sender<SystemServerEvent>,

    /// The project to operate on
    pub work_dir: String,

    /// Whether to use the demo config. This will fake the installation process.
    pub is_demo: bool,
}

pub(crate) type SharedSystemState = Arc<Mutex<EngineInternalState>>;

impl SystemEngine {
    /// Creates a new SystemEngine instance.
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

        let project = Project::load(work_dir.clone().into());
        let project = match project {
            Ok(p) => p,
            Err(e) => {
                error!("Failed to load project: {}", e);
                exit(1);
            }
        };

        let initial_state = EngineInternalState {
            state: system::SystemState::Idle,
            project,
            logs: vec![],
        };

        Self {
            state: Arc::new(Mutex::new(initial_state)),
            event_sender: tx,
            work_dir,
            is_demo,
        }
    }

    /// Updates the system state and broadcasts the change to all listeners
    async fn update_state(&self, new_state: SystemState) {
        let mut state = self.state.lock().await;
        state.state = new_state;
        self.broadcast_state(&state.state);
    }

    /// The central command processor. It takes a command from a client,
    /// modifies the state, and broadcasts events.
    pub async fn handle_command(&self, command: SystemClientCommand) {
        match command {
            SystemClientCommand::SwitchConfig => self.switch_config().await,
            SystemClientCommand::DevReset => self.dev_reset().await,
            SystemClientCommand::Reboot => self.reboot_system_command().await,
        }
    }

    async fn reboot_system_command(&self) {
        if self.is_demo {
            info!("Reboot command received in demo mode. Would trigger system reboot...");
            return;
        }

        info!("Reboot command received. Triggering system reboot...");
        if let Err(e) = reboot_system() {
            error!("Failed to reboot system: {:?}", e);
            let _ = self.event_sender.send(SystemServerEvent::Error(format!(
                "Failed to reboot: {:?}",
                e
            )));
        }
    }

    /// Broadcasts the current state.
    fn broadcast_state(&self, state: &SystemState) {
        let event = SystemServerEvent::StateChanged(state.clone());
        // We don't care if there are no listeners, so we ignore the error.
        let _ = self.event_sender.send(event);
    }

    async fn switch_config(&self) {
        let state_arc = self.state.clone();
        let sender_clone = self.event_sender.clone();

        if self.is_demo {
            self.update_state(SystemState::Switching).await;
            tokio::spawn(async move { fake_switch_config_process(state_arc, sender_clone).await });
        } else {
            let work_dir = {
                let mut state = state_arc.lock().await;

                if !matches!(state.state, SystemState::Idle) {
                    let _ = self.event_sender.send(SystemServerEvent::Error(
                        "Not in correct state to switch config.".to_string(),
                    ));
                    return;
                }

                state.state = SystemState::Switching;
                self.broadcast_state(&state.state);

                self.work_dir.clone()
            };

            tokio::spawn(async move {
                real_switch_config_process(state_arc, sender_clone, work_dir).await;
            });
        }
    }

    async fn dev_reset(&self) {
        let mut state = self.state.lock().await;
        state.state = SystemState::Idle;
        state.logs = vec![];
        self.broadcast_state(&state.state);
    }
}

fn add_and_send_update_log(
    state: &mut EngineInternalState,
    sender: &broadcast::Sender<SystemServerEvent>,
    log: String,
) {
    state.logs.push(log.clone());
    let _ = sender.send(SystemServerEvent::UpdateLog(log));
}

async fn real_switch_config_process(
    state_arc: SharedSystemState,
    sender: broadcast::Sender<SystemServerEvent>,
    work_dir: String,
) {
    info!("Starting real update process");

    let res = switch_config(state_arc.clone(), sender.clone(), &work_dir).await;
    match res {
        Ok(_) => {} // Do nothing
        Err(e) => {
            let err_str = format!("{:?}", e);
            error!("{}", err_str.clone());
            let mut state = state_arc.lock().await;
            add_and_send_update_log(&mut state, &sender, err_str);
            return;
        }
    };

    {
        let mut state_guard = state_arc.lock().await;
        state_guard.state = SystemState::Idle;
        debug!("Apply config process completed successfully.");
        let final_event = SystemServerEvent::StateChanged(state_guard.state.clone());
        let _ = sender.send(final_event);
    }
}

/// This function simulates the entire update process, sending events
/// to the provided sender to update clients.
async fn fake_switch_config_process(
    state_arc: SharedSystemState,
    sender: broadcast::Sender<SystemServerEvent>,
) {
    for step_name in SystemConfigSwitchStepName::VARIANTS {
        {
            let mut state_guard = state_arc.lock().await;
            add_and_send_update_log(
                &mut state_guard,
                &sender,
                format!("Starting step: {}", step_name),
            );
        }

        let sleep_duration_secs = match step_name {
            SystemConfigSwitchStepName::Build => 4,
            SystemConfigSwitchStepName::PostSwitch => 3,
            _ => 1,
        };
        tokio::time::sleep(Duration::from_secs(sleep_duration_secs)).await;

        {
            let mut state_guard = state_arc.lock().await;
            add_and_send_update_log(&mut state_guard, &sender, "...".to_string());
        }
    }

    tokio::time::sleep(Duration::from_secs(1)).await;

    {
        let mut state_guard = state_arc.lock().await;
        state_guard.state = SystemState::Idle;
        debug!("Fake installation process completed successfully.");
        let final_event = SystemServerEvent::StateChanged(state_guard.state.clone());
        let _ = sender.send(final_event);
    }
}
