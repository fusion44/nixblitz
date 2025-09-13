use std::{
    sync::{Arc, Mutex, RwLock},
    time::Duration,
};

use iocraft::hooks::State;
use log::{debug, error, info, warn};
use nixblitz_core::{SystemClientCommand, SystemServerEvent, SystemState};
use tokio::{
    net::TcpStream,
    sync::{
        mpsc::{UnboundedSender, unbounded_channel},
        oneshot, watch,
    },
};

use futures_util::{SinkExt, StreamExt, stream::SplitSink};

use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite::Message};

use crate::tui_shared::{
    ConnectionStatus, PopupData, PopupDataState, ShowPopupState, SwitchLogsState,
};

#[derive(Clone)]
pub(crate) struct TuiSystemEngineConnection {
    pub command_sender: Arc<RwLock<Option<UnboundedSender<SystemClientCommand>>>>,
}

impl TuiSystemEngineConnection {
    pub fn new() -> Self {
        Self {
            command_sender: Arc::new(RwLock::new(None)),
        }
    }

    pub fn send_command(&self, command: SystemClientCommand) {
        // Lock the sender for reading.
        if let Some(sender) = self.command_sender.read().unwrap().as_ref() {
            debug!("[TUI] -> Sending command: {:?}", command);
            if let Err(e) = sender.send(command) {
                error!("[TUI] Error: Failed to send command: {}", e);
            }
        } else {
            error!("[TUI] Error: Cannot send command, not connected.");
        }
    }
}

pub(crate) async fn connect_and_manage(
    engine: Arc<Mutex<TuiSystemEngineConnection>>,
    url: &str,
    mut state: State<SystemState>,
    switch_logs: SwitchLogsState,
    show_popup: ShowPopupState,
    popup_data: PopupDataState,
    mut shutdown_rx: oneshot::Receiver<()>,
    connection_status_tx: watch::Sender<ConnectionStatus>,
) {
    let s_logs = switch_logs.clone();
    let url = url.to_string();
    tokio::spawn({
        let s_logs = s_logs.clone();
        let show_popup = show_popup.clone();
        let popup_data = popup_data.clone();

        async move {
            let mut attempts = 0;
            loop {
                if attempts >= 3 {
                    warn!("[System] Maximum connection attempts reached.");
                    let _ = connection_status_tx.send(ConnectionStatus::Disconnected);
                    return;
                }
                attempts += 1;
                debug!("[System] Attempting to connect to {}...", url.clone());
                let _ = connection_status_tx.send(ConnectionStatus::Connecting);
                match connect_async(url.clone()).await {
                    Ok((ws_stream, _response)) => {
                        debug!("[System] ✅ WebSocket connection successful!");
                        let _ = connection_status_tx.send(ConnectionStatus::Connected);
                        let (tx, mut rx) = unbounded_channel::<SystemClientCommand>();
                        engine
                            .lock()
                            .expect("BUG: engine lock poisoned")
                            .command_sender
                            .write()
                            .unwrap()
                            .replace(tx);

                        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

                        loop {
                            tokio::select! {
                                _ = &mut shutdown_rx => {
                                    info!("[System] Shutdown signal received. Closing connection.");
                                    if let Err(e) = ws_sender.close().await {
                                        error!("[System] Error closing websocket: {:?}", e);
                                    }
                                    return;
                                }
                                message_option = ws_receiver.next() => {
                                    match message_option {
                                        Some(Ok(msg)) => {
                                            info!("[Server] -> Received text message {:?}.", msg);
                                            handle_server_message(
                                                msg,
                                                &mut state,
                                                s_logs.clone(),
                                                show_popup.clone(),
                                                popup_data.clone(),
                                            );
                                        }
                                        Some(Err(e)) => {
                                            error!("[System] ❌ WebSocket read error: {:?}", e);
                                            break;
                                        }
                                        None => {
                                            warn!("[System] WebSocket connection closed by server.");
                                            break;
                                        }
                                    }
                                },
                                Some(command) = rx.recv() => {
                                    if handle_client_command(command, &mut ws_sender).await.is_err() {
                                        error!("[System] ❌ WebSocket write error. Breaking connection.");
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!(
                            "[System] ❌ Failed to connect: {}. Retrying in 1 seconds...",
                            e
                        );
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                }

                engine
                    .lock()
                    .expect("BUG: engine lock poisoned")
                    .command_sender
                    .write()
                    .unwrap()
                    .take();
                warn!("[System] Connection lost.");
                let _ = connection_status_tx.send(ConnectionStatus::Disconnected);
                if shutdown_rx.try_recv().is_ok() {
                    info!(
                        "[System] Shutdown signal received during disconnected state. Exiting task."
                    );
                    return;
                }
            }
        }
    });
}

/// Processes a message received from the WebSocket server.
fn handle_server_message(
    msg: Message,
    state: &mut State<SystemState>,
    logs_state: SwitchLogsState,
    show_popup: ShowPopupState,
    popup_data: PopupDataState,
) {
    match msg {
        Message::Text(text) => {
            debug!("[Server] -> Received text message {:?}.", text);
            if let Ok(event) = serde_json::from_str::<SystemServerEvent>(&text) {
                match event {
                    SystemServerEvent::StateChanged(new_state) => {
                        match new_state {
                            SystemState::Idle => (),
                            SystemState::Switching => {
                                info!("Switching config...");
                                *show_popup.lock().unwrap().write() = true;
                                *popup_data.lock().unwrap().write() = Some(PopupData::Update);
                                *logs_state.lock().unwrap().write() = Vec::new();
                            }
                            SystemState::UpdateFailed(_) => todo!(),
                            SystemState::UpdateSucceeded => {
                                info!("Config switched successfully.");
                                *show_popup.lock().unwrap().write() = false;
                                *popup_data.lock().unwrap().write() = None;
                            }
                        }
                        *state.write() = new_state;
                    }
                    SystemServerEvent::UpdateLog(log_line) => {
                        let mut log = logs_state.lock().unwrap();
                        log.write().push(log_line);
                    }
                    _ => {
                        debug!("Unhandled event: {:?}", text);
                    }
                }
            }
        }
        Message::Binary(_) => {
            debug!("[Server] -> Received binary message (ignored).");
        }
        Message::Ping(_) => {
            debug!("[Server] -> Received Ping (updating last_pong_time)");
            // *last_pong_time = Instant::now();
        }
        Message::Pong(_) => {
            debug!("[Server] -> Received Pong (updating last_pong_time)");
            // *last_pong_time = Instant::now();
        }
        Message::Close(c) => {
            debug!("[Server] -> Received close frame: {:?}", c);
        }
        Message::Frame(_) => {
            debug!("[Server] -> Received a raw frame (should not happen with tokio-tungstenite).");
        }
    }
}

/// Serializes and sends a command from the UI to the WebSocket server.
async fn handle_client_command(
    command: SystemClientCommand,
    ws_sender: &mut SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
) -> Result<(), tokio_tungstenite::tungstenite::Error> {
    match serde_json::to_string(&command) {
        Ok(json_str) => {
            debug!("[System] Sending command as JSON: {}", json_str);
            Ok(ws_sender.send(Message::Text(json_str.into())).await?)
        }
        Err(e) => {
            error!("[System] Error: Failed to serialize client command: {}", e);
            Ok(())
        }
    }
}
