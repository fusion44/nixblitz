use nixblitz_core::*;
use nixblitz_system::installer::{get_process_list, get_system_info, perform_system_check};
use std::sync::Arc;
use tokio::sync::{Mutex, broadcast};

/// The state of the installation process.
pub struct InstallEngine {
    /// The single source of truth for the state of the installation.
    pub state: InstallState,

    /// The broadcast channel for sending events to clients.
    pub event_sender: broadcast::Sender<ServerEvent>,
}

pub type SharedInstallEngine = Arc<Mutex<InstallEngine>>;

impl InstallEngine {
    /// Creates a new InstallEngine instance.
    pub fn new() -> Self {
        let (tx, _rx) = broadcast::channel(100);

        Self {
            state: InstallState::Idle,
            event_sender: tx,
        }
    }

    /// The central command processor. It takes a command from a client,
    /// modifies the state, and broadcasts events.
    pub async fn handle_command(&mut self, command: ClientCommand) {
        match command {
            ClientCommand::PerformSystemCheck => self.perform_system_check().await,
            ClientCommand::GetSystemSummary => self.get_system_summary().await,
            ClientCommand::GetProcessList => self.get_process_list().await,
            ClientCommand::StartInstallation => self.start_installation().await,
            _ => {
                let _ = self
                    .event_sender
                    .send(ServerEvent::Error("Command not implemented".into()));
            }
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

    async fn start_installation(&mut self) {
        if let InstallState::SystemCheckCompleted(ref result) = self.state {
            if !result.is_compatible {
                let _ = self.event_sender.send(ServerEvent::Error(
                    "Cannot install on an incompatible system.".into(),
                ));
                return;
            }

            self.state = InstallState::Installing;
            self.broadcast_state();
        } else {
            let _ = self.event_sender.send(ServerEvent::Error(
                "System check must be performed before installation.".into(),
            ));
        }
    }
}
