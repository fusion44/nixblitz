use std::sync::{Arc, RwLock};

use dioxus::prelude::*;
use dioxus_logger::tracing;
use nixblitz_core::{SystemClientCommand, SystemServerEvent, SystemState};

use crate::components::{Button, Configurator};
use futures::{
    SinkExt, StreamExt,
    channel::mpsc::{self, UnboundedSender},
    future,
};
use gloo_net::websocket::{Message, futures::WebSocket};

type SystemStateSignal = Signal<Arc<RwLock<Option<SystemState>>>>;
type SystemClientCommandSignal = Signal<Option<UnboundedSender<SystemClientCommand>>>;
type SystemLogsSignal = Signal<Vec<String>>;

#[cfg(not(feature = "server"))]
fn get_ws_url() -> String {
    #[cfg(debug_assertions)]
    {
        const DEV_SYS_WS_URL: &str = "ws://127.0.0.1:3000/ws";
        let url = std::env::var("NIXBLITZ_SYSTEM_WS_OVERRIDE")
            .unwrap_or_else(|_| DEV_SYS_WS_URL.to_string());
        tracing::info!("Using development WebSocket URL: {}", &url);
        url
    }

    #[cfg(not(debug_assertions))]
    {
        use web_sys::window;

        let window = window().expect("Must be in a browser environment");
        let location = window.location();
        let protocol = location.protocol().expect("Failed to get protocol");
        let host = location.host().expect("Failed to get host");

        let ws_protocol = if protocol == "https:" { "wss:" } else { "ws:" };

        let url = format!("{}//{}/system/ws", ws_protocol, host);
        tracing::info!("Using production WebSocket URL: {}", &url);
        url
    }
}

#[cfg(feature = "server")]
fn get_ws_url() -> String {
    tracing::info!("Using empty WebSocket URL on server");
    String::new()
}

#[component]
pub fn Config() -> Element {
    let system_state: SystemStateSignal = use_signal(|| Arc::new(RwLock::new(None)));
    let command_sender: SystemClientCommandSignal = use_signal(|| None);
    let logs: SystemLogsSignal = use_signal(Vec::new);
    let mut url: Signal<String> = use_signal(String::new);
    let mut show_logs_modal = use_signal(|| false);

    use_effect(move || {
        url.set(get_ws_url());
    });

    use_effect(move || {
        let url_for_task = url();
        if !url_for_task.is_empty() {
            tracing::debug!("Effect is running. Connecting to: {}", &url_for_task);
            spawn_connection_task(system_state, logs, command_sender, url_for_task);
        } else {
            tracing::debug!("Effect is running. No URL to connect to.");
        }
    });

    let state_lock = system_state.read();
    let maybe_state = state_lock.read().expect("BUG: state lock poisoned");

    let state_component = rsx! {
        div {
            id: "state_display",
            class: "px-4 py-2 bg-zinc-800 rounded-md text-zinc-300 text-sm",
            match &*maybe_state {
                Some(SystemState::Idle) => "Status: Idle",
                Some(SystemState::Switching) => "Status: Switching...",
                Some(SystemState::UpdateFailed(e)) => "Status: Failed ({e})",
                Some(SystemState::UpdateSucceeded) => "Status: Success",
                None => "Status: Connecting...",
            }
        }
    };

    let state_button = rsx! {
        Button {
            on_click: move |_| {
                if let Some(sender) = command_sender.read().as_ref() {
                    let _ = sender.unbounded_send(SystemClientCommand::SwitchConfig);
                }
            },
            loading: match &*maybe_state {
                Some(SystemState::Switching) => true,
                _ => false,
            },
            "Switch Config"
        }
    };

    rsx! {
        div { id: "hero",
            header { class: "w-full border-b border-zinc-800 sticky top-0 backdrop-blur-sm z-10",
                div { class: "max-w-screen-xl mx-auto p-4 flex justify-between items-center",
                    div { class: "flex items-center space-x-4",
                        div { class: "w-8 h-8 bg-gradient-to-br from-blue-500 to-purple-600 rounded-md flex items-center justify-center font-bold",
                            "N"
                        }
                        h1 { class: "text-xl font-bold text-zinc-100", "NixBlitz" }
                    }

                    // State display and log button
                    div { class: "flex items-center space-x-2",
                        {state_component}
                        button {
                            title: "Show Logs",
                            class: "p-2 rounded-md bg-zinc-800 text-zinc-400 hover:bg-zinc-700 hover:text-white transition-colors",
                            onclick: move |_| show_logs_modal.set(true),
                            svg {
                                xmlns: "http://www.w3.org/2000/svg",
                                width: "16",
                                height: "16",
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "2",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                path { d: "M8 21h12a2 2 0 0 0 2-2v-2H10v2a2 2 0 1 1-4 0V5a2 2 0 1 0-4 0v3h4" }
                                path { d: "M19 17V5a2 2 0 0 0-2-2H4a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h12" }
                                path { d: "M15 9h-5" }
                                path { d: "M15 13h-5" }
                            }
                        }
                    }

                    div { class: "flex items-center space-x-2", {state_button} }
                }
            }

            main { class: "w-full p-4", Configurator {} }

            // Log Modal
            if *show_logs_modal.read() {
                div {
                    class: "fixed inset-0 bg-black/60 z-50 flex items-center justify-center",
                    onclick: move |_| show_logs_modal.set(false),
                    div {
                        class: "bg-zinc-900 border border-zinc-700 rounded-lg shadow-xl w-full max-w-4xl h-3/4 flex flex-col",
                        onclick: |evt| evt.stop_propagation(), // Prevent clicks inside modal from closing it
                        div { class: "flex justify-between items-center p-4 border-b border-zinc-700",
                            h3 { class: "font-bold text-lg text-zinc-100",
                                if logs.read().is_empty() {
                                    "Logs (Empty)"
                                } else {
                                    "Logs"
                                }
                            }
                            button {
                                class: "text-zinc-500 hover:text-white transition-colors",
                                onclick: move |_| show_logs_modal.set(false),
                                svg {
                                    xmlns: "http://www.w3.org/2000/svg",
                                    width: "24",
                                    height: "24",
                                    view_box: "0 0 24 24",
                                    fill: "none",
                                    stroke: "currentColor",
                                    stroke_width: "2",
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    line {
                                        x1: "18",
                                        y1: "6",
                                        x2: "6",
                                        y2: "18",
                                    }
                                    line {
                                        x1: "6",
                                        y1: "6",
                                        x2: "18",
                                        y2: "18",
                                    }
                                }
                            }
                        }
                        // Modal Body
                        div { class: "p-4 flex-grow overflow-y-auto",
                            if logs.read().is_empty() {
                                div { class: "text-zinc-400 text-center h-full flex items-center justify-center",
                                    "No log messages to display."
                                }
                            } else {
                                pre { class: "text-sm text-zinc-300 whitespace-pre-wrap font-mono",
                                    for line in logs.read().iter() {
                                        "{line}\n"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn spawn_connection_task(
    mut state: SystemStateSignal,
    mut logs: SystemLogsSignal,
    mut sender: SystemClientCommandSignal,
    url: String,
) {
    spawn(async move {
        let (tx, mut rx) = mpsc::unbounded::<SystemClientCommand>();
        sender.set(Some(tx));

        tracing::debug!("Establishing WebSocket connection to {}", url.clone());

        let ws = match WebSocket::open(&url) {
            Ok(ws) => ws,
            Err(e) => {
                tracing::error!("Failed to open WebSocket object: {}", e);
                return;
            }
        };
        tracing::debug!("WebSocket connection object created, attempting to connect...");

        let (mut ws_writer, mut ws_reader) = ws.split();

        let incoming_loop = async {
            loop {
                match ws_reader.next().await {
                    Some(Ok(Message::Text(text))) => {
                        if let Ok(event) = serde_json::from_str::<SystemServerEvent>(&text) {
                            tracing::debug!(?event, "Received event from server");
                            match event {
                                SystemServerEvent::StateChanged(new_state) => {
                                    // if let SystemState::Idle = &new_state {
                                    //     *logs.write() = Vec::new(); // Clear old logs
                                    // }

                                    let state_lock = state.write();
                                    let mut data =
                                        state_lock.write().expect("BUG: state lock poisoned");
                                    *data = Some(new_state);
                                }
                                SystemServerEvent::UpdateLog(log_line) => {
                                    logs.write().push(log_line);
                                }
                                _ => {
                                    tracing::debug!("Unhandled event: {:?}", event);
                                }
                            }
                        }
                    }
                    Some(Ok(Message::Bytes(_))) => {
                        tracing::debug!("Received binary message, ignoring.");
                    }
                    Some(Err(e)) => {
                        tracing::error!("WebSocket connection error: {}", e);
                        break;
                    }
                    None => {
                        tracing::warn!("WebSocket stream closed. Is the server running?");
                        break;
                    }
                }
            }
        };

        let outgoing_loop = async {
            while let Some(command) = rx.next().await {
                if let Ok(payload) = serde_json::to_string(&command) {
                    if ws_writer.send(Message::Text(payload)).await.is_err() {
                        // This error means the connection is closed.
                        break;
                    }
                }
            }
        };

        // Needs to be pinned to be able to be used with future::select
        let pinned_incoming = Box::pin(incoming_loop);
        let pinned_outgoing = Box::pin(outgoing_loop);

        future::select(pinned_incoming, pinned_outgoing).await;
        tracing::info!("WebSocket connection closed.");
    });
}
