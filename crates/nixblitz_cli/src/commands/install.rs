use std::{
    io, panic,
    path::Path,
    sync::{Arc, Mutex, RwLock},
    time::Duration,
};

use crossterm::{
    cursor::{Hide, Show},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use error_stack::{Result, ResultExt};
use futures_util::{SinkExt, StreamExt, stream::SplitSink};
use iocraft::prelude::*;
use log::{debug, error, info, warn};
use nixblitz_core::{DiskoInstallStep, InstallClientCommand, InstallServerEvent, InstallState};
use tokio::{
    net::TcpStream,
    sync::mpsc::{UnboundedSender, unbounded_channel},
};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite::Message};

use crate::{
    errors::CliError,
    tui_components::{
        Configurator,
        install::{
            InstallDiskSelection, InstallSuccess, Installing, PreInstallConfirm,
            SystemCheckDisplay, Welcome,
        },
        utils::load_or_create_project,
    },
};

/// Manages the connection state and communication with the WebSocket task.
/// This struct can be safely cloned and shared between threads.
#[derive(Clone)]
pub struct EngineConnection {
    /// A channel to send commands FROM the UI TO the WebSocket task.
    /// This is set by the connection task itself.
    pub command_sender: Arc<RwLock<Option<UnboundedSender<InstallClientCommand>>>>,
}

impl EngineConnection {
    /// Create a new, uninitialized connection service.
    pub fn new() -> Self {
        Self {
            command_sender: Arc::new(RwLock::new(None)),
        }
    }

    /// Sends a command to the engine's WebSocket task.
    pub fn send_command(&self, command: InstallClientCommand) {
        // Lock the sender for reading.
        if let Some(sender) = self.command_sender.read().unwrap().as_ref() {
            debug!("[UI] -> Sending command: {:?}", command);
            if let Err(e) = sender.send(command) {
                error!("[UI] Error: Failed to send command: {}", e);
            }
        } else {
            error!("[UI] Error: Cannot send command, not connected.");
        }
    }
}

pub async fn start_install_app(work_dir: &Path) -> Result<(), CliError> {
    let project = match load_or_create_project(work_dir, true).await? {
        Some(p) => p,
        None => return Ok(()),
    };

    let project = Arc::new(Mutex::new(project));

    fn restore_terminal() {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, Show);
    }

    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        restore_terminal();
        original_hook(panic_info);
    }));

    enable_raw_mode().change_context(CliError::UnableToStartTui)?;
    execute!(io::stdout(), EnterAlternateScreen, Hide)
        .change_context(CliError::UnableToStartTui)?;

    let result = tokio::task::spawn(async move {
        let _ = element! {
            ContextProvider(value: Context::owned(project)) {
                InstallerTuiApp
            }
        }
        .render_loop()
        .await;
    })
    .await;

    restore_terminal();

    if let Err(e) = result {
        error!("Render loop panicked: {:?}", e);
        return Err(CliError::GenericError("Render loop failed".to_string()).into());
    }

    Ok(())
}

type InstallStepsState = Arc<Mutex<State<Vec<DiskoInstallStep>>>>;
type InstallLogsState = Arc<Mutex<State<Vec<String>>>>;

#[component]
fn InstallerTuiApp(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let (width, height) = hooks.use_terminal_size();
    let mut system = hooks.use_context_mut::<SystemContext>();
    let mut show_help = hooks.use_state(|| false);
    let mut should_exit = hooks.use_state(|| false);
    let mut connected = hooks.use_state(|| false);
    let engine = hooks.use_state(|| Arc::new(Mutex::new(EngineConnection::new())));
    let mut state = hooks.use_state(|| InstallState::Idle);
    let install_steps: InstallStepsState = Arc::new(Mutex::new(hooks.use_state(Vec::new)));
    let install_logs: InstallLogsState = Arc::new(Mutex::new(hooks.use_state(Vec::new)));

    hooks.use_future({
        let install_logs_clone = install_logs.clone();
        let install_steps_clone = install_steps.clone();
        let e = engine.read().clone();
        async move {
            connect_and_manage(
                e,
                "ws://127.0.0.1:3000/ws",
                &mut state,
                install_steps_clone,
                install_logs_clone,
            )
            .await;
            connected.set(true);
        }
    });

    hooks.use_terminal_events({
        move |event| {
            if *state.read() == InstallState::UpdateConfig {
                // Configurator GUI handles all key events
                return;
            }

            match event {
                TerminalEvent::Key(KeyEvent { code, kind, .. })
                    if kind != KeyEventKind::Release =>
                {
                    match code {
                        KeyCode::Char('q') => should_exit.set(true),
                        KeyCode::Char('?') => {
                            if !show_help.get() {
                                show_help.set(true)
                            } else {
                                show_help.set(false)
                            }
                        }
                        KeyCode::Char('r') => {
                            let engine = engine.read().clone();
                            engine
                                .lock()
                                .expect("BUG: engine lock poisoned")
                                .send_command(InstallClientCommand::DevReset);
                        }
                        KeyCode::Enter => match *state.read() {
                            InstallState::Idle => advance(
                                engine.read().clone(),
                                InstallClientCommand::PerformSystemCheck,
                            ),
                            InstallState::SystemCheckCompleted(_) => {
                                advance(engine.read().clone(), InstallClientCommand::UpdateConfig)
                            }
                            _ => {
                                debug!("Unhandled advance state: {:?}", state);
                            }
                        },
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    });

    if should_exit.get() {
        system.exit();
    }

    let body = match &*state.read() {
        InstallState::Idle => element! {
            Welcome(connected: *connected.read())
        }
        .into_any(),
        InstallState::PerformingCheck => element! {
            Text(content: "Checking system...")
        }
        .into_any(),
        InstallState::SystemCheckCompleted(result) => element! {
            SystemCheckDisplay(result: result.clone())
        }
        .into_any(),
        InstallState::UpdateConfig => {
            let on_submit = Some(iocraft::Handler::from(move |_| {
                advance(
                    engine.read().clone(),
                    InstallClientCommand::UpdateConfigFinished,
                );
            }));
            element! {
                Configurator(on_submit)
            }
            .into_any()
        }
        InstallState::SelectInstallDisk(disks) => {
            let on_select = Some(iocraft::Handler::from(move |disk| {
                info!("Selected disk: {}", disk);
                advance(
                    engine.read().clone(),
                    InstallClientCommand::InstallDiskSelected(disk),
                );
            }));
            element! {
                InstallDiskSelection (
                    disks: disks.clone(),
                    on_select,
                )
            }
            .into_any()
        }
        InstallState::PreInstallConfirm(data) => {
            let on_confirm = iocraft::Handler::from(move |result: bool| {
                if result {
                    advance(
                        engine.read().clone(),
                        InstallClientCommand::StartInstallation,
                    );
                }
            });
            let data = data.clone();
            element! {
                PreInstallConfirm(data, on_confirm)
            }
        }
        .into_any(),
        InstallState::Installing(_) => {
            let steps = install_steps.lock().unwrap().read().clone();
            let logs = install_logs.lock().unwrap().read().clone();
            element! {
                Installing(steps, logs)
            }
        }
        .into_any(),
        InstallState::InstallSucceeded(final_steps) => {
            let on_reboot = move |_| {
                advance(engine.read().clone(), InstallClientCommand::Reboot);
            };

            let final_steps = final_steps.clone();
            let final_logs = install_logs.lock().unwrap().read().clone();
            element! {
                InstallSuccess(on_reboot, final_steps, final_logs)
            }
        }
        .into_any(),
        state => {
            debug!("InstallState::Unknown");
            let state_str = format!("State not implemented {:?}", state);
            element! {
                Text (content: state_str)
            }
            .into_any()
        }
    };

    element! {
        View(
            width, height,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
        ){
            View(flex_direction: FlexDirection::Column) {
                #(body)
            }
        }
    }
}

fn advance(engine: Arc<Mutex<EngineConnection>>, command: InstallClientCommand) {
    engine
        .lock()
        .expect("BUG: engine lock poisoned")
        .send_command(command);
}

/// Establishes and manages the WebSocket connection in a background task.
pub async fn connect_and_manage(
    engine: Arc<Mutex<EngineConnection>>,
    url_str: &str,
    state: &mut State<InstallState>,
    install_steps_state: InstallStepsState,
    install_logs_state: InstallLogsState,
) {
    let url = url_str.to_string();

    let mut state = *state;
    tokio::spawn({
        let steps_state = install_steps_state.clone();
        let logs_state = install_logs_state.clone();
        async move {
            loop {
                debug!("[System] Attempting to connect to {}...", url);
                match connect_async(url.clone()).await {
                    Ok((ws_stream, _response)) => {
                        debug!("[System] ✅ WebSocket connection successful!");
                        let (tx, mut rx) = unbounded_channel::<InstallClientCommand>();

                        engine
                            .lock()
                            .expect("BUG: engine lock poisoned")
                            .command_sender
                            .write()
                            .unwrap()
                            .replace(tx);

                        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

                        loop {
                            let steps_state = steps_state.clone();
                            let logs_state = logs_state.clone();

                            tokio::select! {
                                Some(Ok(msg)) = ws_receiver.next() => {
                                    handle_server_message(
                                        msg,
                                        &mut state,
                                        steps_state,
                                        logs_state,
                                    );
                                },

                                Some(command) = rx.recv() => {
                                    if handle_client_command(command, &mut ws_sender).await.is_err() {
                                        break;
                                    }
                                },

                                else => {
                                    debug!("[System] A channel was closed. Breaking connection loop.");
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!(
                            "[System] ❌ Failed to connect: {}. Retrying in 5 seconds...",
                            e
                        );
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
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    });
}

/// Processes a message received from the WebSocket server.
fn handle_server_message(
    msg: Message,
    state: &mut State<InstallState>,
    install_steps_state: InstallStepsState,
    install_logs_state: InstallLogsState,
) {
    match msg {
        Message::Text(text) => {
            debug!("[Server] -> Received text message {:?}.", text);
            if let Ok(event) = serde_json::from_str::<InstallServerEvent>(&text) {
                match event {
                    InstallServerEvent::StateChanged(new_state) => {
                        if let InstallState::Installing(steps) = &new_state {
                            *install_steps_state.lock().unwrap().write() = steps.clone();
                            *install_logs_state.lock().unwrap().write() = Vec::new(); // Clear old logs
                        }

                        state.set(new_state.clone());
                    }
                    InstallServerEvent::InstallStepUpdate(updated_step) => {
                        let mut install_steps = install_steps_state.lock().unwrap();
                        let mut install_steps = install_steps.write();

                        if let Some(step) = install_steps
                            .iter_mut()
                            .find(|s| s.name == updated_step.name)
                        {
                            *step = updated_step;
                        }
                    }
                    InstallServerEvent::InstallLog(log_line) => {
                        install_logs_state.lock().unwrap().write().push(log_line);
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
            debug!("[Server] -> Received Ping (ignoring)");
        }
        Message::Pong(_) => {
            debug!("[Server] -> Received Pong (ignoring)");
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
    command: InstallClientCommand,
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
