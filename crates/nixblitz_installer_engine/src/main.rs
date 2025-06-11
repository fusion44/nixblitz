mod engine;

use crate::engine::{InstallEngine, SharedInstallEngine};
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
use nixblitz_core::{ClientCommand, ServerEvent};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::{Mutex, broadcast};
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "nixblitz_installer_engine=debug,tower_http=info".into()),
        )
        .init();
    tracing::debug!("Starting installer engine...");

    let shared_engine: SharedInstallEngine = Arc::new(Mutex::new(InstallEngine::new()));
    let cors = CorsLayer::new().allow_origin(Any {});

    let app = Router::new()
        .route("/ws", get(websocket_handler))
        .with_state(shared_engine)
        .layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("Server listening on {}", addr);

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
    tracing::debug!("New WebSocket client connected.");

    let (mut ws_sender, ws_receiver) = socket.split();

    {
        // send the current state to the new client
        let engine_guard = engine.lock().await;
        let current_state = engine_guard.state.clone();

        tracing::debug!(?current_state, "Sending initial state to new client");

        let event = ServerEvent::StateChanged(current_state);
        let payload =
            serde_json::to_string(&event).expect("Failed to serialize initial state event.");

        if ws_sender.send(Message::Text(payload.into())).await.is_err() {
            tracing::warn!("Client disconnected before initial state could be sent.");
            return;
        }
    }

    let broadcast_rx = engine.lock().await.event_sender.subscribe();

    let outgoing_task = tokio::spawn(handle_outgoing_messages(ws_sender, broadcast_rx));
    let incoming_task = tokio::spawn(handle_incoming_messages(ws_receiver, engine.clone()));

    tokio::select! {
        _ = outgoing_task => tracing::debug!("Outgoing task finished."),
        _ = incoming_task => tracing::debug!("Incoming task finished."),
    }

    tracing::info!("WebSocket client disconnected.");
}

async fn handle_outgoing_messages(
    mut sender: SplitSink<WebSocket, Message>,
    mut broadcast_rx: broadcast::Receiver<ServerEvent>,
) {
    while let Ok(event) = broadcast_rx.recv().await {
        let payload = serde_json::to_string(&event).expect("Failed to serialize server event.");

        if sender.send(Message::Text(payload.into())).await.is_err() {
            tracing::debug!("Client disconnected. Closing outgoing message loop.");
            break;
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
                    tracing::debug!(?command, "Received command from client");
                    engine.lock().await.handle_command(command).await;
                }
                Err(e) => {
                    tracing::error!("Failed to deserialize client message: {}", e);
                }
            }
        }
    }
}
