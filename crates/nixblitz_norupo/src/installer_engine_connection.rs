use dioxus_signals::{Readable, Signal};
use futures::channel::mpsc::UnboundedSender;
use nixblitz_core::{InstallClientCommand, InstallState};
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct EngineConnection {
    /// The current state of the installation process received from the server.
    /// Wrapped in a Signal so that components will automatically re-render when it changes.
    pub install_state: Signal<Arc<RwLock<Option<InstallState>>>>,

    /// A channel to send commands FROM the UI TO the WebSocket task.
    pub command_sender: Signal<Option<UnboundedSender<InstallClientCommand>>>,
}

impl EngineConnection {
    /// Create a new, uninitialized connection service.
    pub fn new() -> Self {
        Self {
            install_state: Signal::new(Arc::new(RwLock::new(None))),
            command_sender: Signal::new(None),
        }
    }

    /// Sends commands to the engine.
    pub fn send_command(&self, command: InstallClientCommand) {
        if let Some(sender) = self.command_sender.read().as_ref() {
            if let Err(e) = sender.unbounded_send(command) {
                println!("Failed to send command: {}", e);
            }
        } else {
            println!("Cannot send command: not connected.");
        }
    }
}
