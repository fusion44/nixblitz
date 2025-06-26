use dioxus_signals::{Readable, Signal};
use futures::channel::mpsc::UnboundedSender;
use nixblitz_core::{SystemClientCommand, SystemState};
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct SystemEngineConnection {
    /// The current state of the system as received from the server.
    /// Wrapped in a Signal so that components will automatically re-render when it changes.
    pub system_state: Signal<Arc<RwLock<Option<SystemState>>>>,

    /// A channel to send commands FROM the UI TO the WebSocket task.
    pub command_sender: Signal<Option<UnboundedSender<SystemClientCommand>>>,
}

impl SystemEngineConnection {
    /// Create a new, uninitialized connection service.
    pub fn new() -> Self {
        Self {
            system_state: Signal::new(Arc::new(RwLock::new(None))),
            command_sender: Signal::new(None),
        }
    }

    /// Sends commands to the engine.
    pub fn send_command(&self, command: SystemClientCommand) {
        if let Some(sender) = self.command_sender.read().as_ref() {
            if let Err(e) = sender.unbounded_send(command) {
                println!("Failed to send command: {}", e);
            }
        } else {
            println!("Cannot send command: not connected.");
        }
    }
}
