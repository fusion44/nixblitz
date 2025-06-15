mod engine;

use crate::engine::InstallEngine;
use axum::{
    Router,
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::IntoResponse,
    routing::get,
};
use futures::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};
use log::{debug, error, info, warn};
use nixblitz_core::{ClientCommand, ServerEvent};
use std::io::Write;
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};

type SharedInstallEngine = Arc<InstallEngine>;

const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
const GIT_SHA: &str = match option_env!("VERGEN_GIT_SHA") {
    Some(sha) => sha,
    None => "sha-unknown",
};

fn init_logging() {
    match std::env::var("RUST_LOG_STYLE") {
        Ok(s) if s == "SYSTEMD" => env_logger::builder()
            .format(|buf, record| {
                writeln!(
                    buf,
                    "<{}>{}: {}",
                    match record.level() {
                        log::Level::Error => 3,
                        log::Level::Warn => 4,
                        log::Level::Info => 6,
                        log::Level::Debug => 7,
                        log::Level::Trace => 7,
                    },
                    record.target(),
                    record.args()
                )
            })
            .init(),
        _ => env_logger::init(),
    };
}

#[tokio::main]
async fn main() {
    init_logging();

    debug!("Starting installer engine...");
    info!("Version: {PKG_VERSION}");
    info!("Git SHA: {GIT_SHA}");

    let shared_engine: SharedInstallEngine = Arc::new(InstallEngine::new());
    let cors = CorsLayer::new().allow_origin(Any {});

    let app = Router::new()
        .route("/ws", get(websocket_handler))
        .with_state(shared_engine)
        .layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    debug!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(engine): State<SharedInstallEngine>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, engine))
}

async fn handle_socket(socket: WebSocket, engine: SharedInstallEngine) {
    debug!("New WebSocket client connected.");
    let (mut ws_sender, ws_receiver) = socket.split();

    {
        let state_guard = engine.state.lock().await;
        let current_state = state_guard.install_state.clone();

        debug!("Sending initial state to new client: {:?}", current_state);

        let event = ServerEvent::StateChanged(current_state);
        let payload =
            serde_json::to_string(&event).expect("Failed to serialize initial state event.");

        if ws_sender.send(Message::Text(payload.into())).await.is_err() {
            warn!("Client disconnected before initial state could be sent.");
            return;
        }
    }

    let broadcast_rx = engine.event_sender.subscribe();
    let outgoing_task = tokio::spawn(handle_outgoing_messages(ws_sender, broadcast_rx));
    let incoming_task = tokio::spawn(handle_incoming_messages(ws_receiver, engine.clone()));

    tokio::select! {
        _ = outgoing_task => debug!("Outgoing task finished."),
        _ = incoming_task => debug!("Incoming task finished."),
    }

    info!("WebSocket client disconnected.");
}

async fn handle_outgoing_messages(
    mut sender: SplitSink<WebSocket, Message>,
    mut broadcast_rx: broadcast::Receiver<ServerEvent>,
) {
    loop {
        match broadcast_rx.recv().await {
            Ok(event) => {
                let payload =
                    serde_json::to_string(&event).expect("Failed to serialize server event.");

                if sender.send(Message::Text(payload.into())).await.is_err() {
                    debug!("Client disconnected. Closing outgoing message loop.");
                    break;
                }
            }
            Err(broadcast::error::RecvError::Lagged(n)) => {
                warn!("Client event stream lagged, skipped {} messages.", n);
                continue;
            }
            Err(broadcast::error::RecvError::Closed) => {
                debug!("Broadcast channel closed. Closing outgoing message loop.");
                break;
            }
        }
    }
}

async fn handle_incoming_messages(
    mut receiver: SplitStream<WebSocket>,
    engine: SharedInstallEngine,
) {
    while let Some(Ok(message)) = receiver.next().await {
        if let Message::Text(text) = message {
            match serde_json::from_str::<ClientCommand>(&text) {
                Ok(command) => {
                    debug!("Received command from client: {:?}", command);
                    engine.handle_command(command).await;
                }
                Err(e) => {
                    error!("Failed to deserialize client message: {}", e);
                }
            }
        }
    }
}
