mod core;
mod engine;

use crate::engine::SystemEngine;
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
use nixblitz_core::{NIXBLITZ_DEMO, SystemClientCommand, SystemServerEvent};
use nixblitz_system::utils::get_env_var;
use std::{
    env,
    io::Write,
    net::{IpAddr, Ipv4Addr},
};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};

type SharedSystemEngine = Arc<SystemEngine>;

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

fn is_demo() -> bool {
    match env::var(NIXBLITZ_DEMO) {
        Ok(v) => v == "1",
        Err(_) => false,
    }
}

#[tokio::main]
async fn main() {
    init_logging();
    let is_demo = is_demo();

    info!("Starting system engine...");
    info!("Version: {PKG_VERSION}");
    info!("Git SHA: {GIT_SHA}");
    info!("Mode: {}", if is_demo { "DEMO" } else { "LIVE" });

    let shared_engine: SharedSystemEngine = Arc::new(SystemEngine::new(is_demo));
    let cors = CorsLayer::new().allow_origin(Any {});

    let app = Router::new()
        .route("/ws", get(websocket_handler))
        .with_state(shared_engine)
        .layer(cors);

    let ip = get_env_var("IP", IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
    let port = get_env_var("PORT", 3000u16);
    let addr = SocketAddr::new(ip, port);
    info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(engine): State<SharedSystemEngine>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, engine))
}

async fn handle_socket(socket: WebSocket, engine: SharedSystemEngine) {
    debug!("New WebSocket client connected.");
    let (mut ws_sender, ws_receiver) = socket.split();

    {
        let state_guard = engine.state.lock().await;
        let current_state = state_guard.state.clone();

        debug!("Sending initial state to new client: {:?}", current_state);

        let event = SystemServerEvent::StateChanged(current_state);
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
    mut broadcast_rx: broadcast::Receiver<SystemServerEvent>,
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
    engine: SharedSystemEngine,
) {
    while let Some(Ok(message)) = receiver.next().await {
        if let Message::Text(text) = message {
            match serde_json::from_str::<SystemClientCommand>(&text) {
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
