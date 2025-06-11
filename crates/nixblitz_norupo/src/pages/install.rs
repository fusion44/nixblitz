use std::sync::{Arc, RwLock};

use dioxus::prelude::*;
use dioxus::prelude::*;
use dioxus_logger::tracing;
use nixblitz_core::{
    InstallState,
    app_option_data::{
        option_data::{GetOptionId, OptionData},
        *,
    },
};

use crate::{
    backend::{get_app_options_wrapper, get_supported_apps_wrapper},
    classes::{buttons, typography::headings},
    components::{
        InstallDiskSelection, Installing, PreInstallConfirm,
        button::Button,
        install::{SystemCheckDisplay, Welcome},
        option_editors::*,
    },
    installer_engine_connection::EngineConnection,
    pages::Config,
};
use futures::{SinkExt, StreamExt, channel::mpsc, future};
use gloo_net::websocket::{Message, futures::WebSocket};
use nixblitz_core::{ClientCommand, ServerEvent};

type InstallStateSignal = Signal<Arc<RwLock<Option<InstallState>>>>;
type ClientCommandSignal = Signal<Option<UnboundedSender<ClientCommand>>>;

#[component]
pub fn Install() -> Element {
    let mut install_state: InstallStateSignal = use_signal(|| Arc::new(RwLock::new(None)));
    let mut command_sender: ClientCommandSignal = use_signal(|| None);

    use_effect(move || {
        spawn_connection_task(install_state, command_sender);
    });

    let state_lock = install_state.read();
    let maybe_state = state_lock.read().expect("BUG: state lock poisoned");

    rsx! {
        div { class: "flex flex-col items-center justify-center text-center space-y-8 h-full w-full",
            h1 { class: "{headings::H1} text-slate-100", "NixBlitz Installer" }

            match &*maybe_state {
                None => {
                    rsx! {
                        p { class: "text-lg text-gray-500 mb-8", "Waiting for connection..." }
                        div { class: "mt-8 w-full min-h-[350px] flex items-center justify-center",
                            div { class: "flex flex-col items-center text-gray-500",
                                div { class: "w-10 h-10 border-4 border-gray-200 border-t-gray-600 rounded-full animate-spin" }
                                p { class: "mt-4 text-lg", "Connecting to Installer Engine..." }
                            }
                        }
                    }
                }
                Some(InstallState::Idle) => {
                    rsx! {
                        Welcome {
                            on_click: move |_| {
                                tracing::debug!("Clicked 'Next' button on welcome screen.");
                                if let Some(sender) = command_sender.read().as_ref() {
                                    let _ = sender.unbounded_send(ClientCommand::PerformSystemCheck);
                                }
                            },
                        }
                    }
                }
                Some(InstallState::PerformingCheck) => {
                    rsx! {
                        Welcome {
                            loading: true,
                            on_click: move |_| {
                                if let Some(sender) = command_sender.read().as_ref() {
                                    let _ = sender.unbounded_send(ClientCommand::PerformSystemCheck);
                                }
                            },
                        }
                    }
                }
                Some(InstallState::SystemCheckCompleted(result)) => {
                    rsx! {
                        SystemCheckDisplay {
                            result: result.clone(),
                            on_click: move |_| {
                                if let Some(sender) = command_sender.read().as_ref() {
                                    let _ = sender.unbounded_send(ClientCommand::UpdateConfig);
                                }
                            },
                        }
                    }
                }
                Some(InstallState::UpdateConfig) => {
                    rsx! {
                        div {
                            "By default, no services will be enabled. Make sure to enable the services you want to use. You can also change the options for the services you enabled. Once you are done, click 'Continue' to continue with the installation."
                        }
                        Config {}
                        Button {
                            on_click: move |evt| {
                                if let Some(sender) = command_sender.read().as_ref() {
                                    let _ = sender.unbounded_send(ClientCommand::UpdateConfigFinished);
                                }
                            },
                            "Continue"
                        }
                    }
                }
                Some(InstallState::SelectInstallDisk(disks)) => {
                    rsx! {
                        InstallDiskSelection {
                            disks: disks.clone(),
                            on_select: move |disk| {
                                tracing::info!("Selected disk: {}", disk);
                                if let Some(sender) = command_sender.read().as_ref() {
                                    let _ = sender.unbounded_send(ClientCommand::InstallDiskSelected(disk));
                                }
                            },
                        }
                    }
                }
                Some(InstallState::PreInstallConfirm(data)) => {
                    let clone1 = data.clone();
                    rsx! {
                        PreInstallConfirm {
                            data: clone1,
                            on_confirm: move |disk| {
                                if let Some(sender) = command_sender.read().as_ref() {
                                    let _ = sender.unbounded_send(ClientCommand::StartInstallation);
                                }
                            },
                        }
                    }
                }
                Some(InstallState::Installing(message)) => {
                    rsx! {
                        Installing { message }
                    }
                }
                Some(_) => {
                    rsx! {
                        h3 { class: "text-xl font-bold mb-2", "State not implemented" }
                    }
                }
            }

            Button {
                on_click: move |evt| {
                    if let Some(sender) = command_sender.read().as_ref() {
                        let _ = sender.unbounded_send(ClientCommand::DevReset);
                    }
                },
                "Reset State"
            }
        }
    }
}

fn spawn_connection_task(
    mut install_state: InstallStateSignal,
    mut command_sender: ClientCommandSignal,
) {
    spawn(async move {
        tracing::debug!("Establishing WebSocket connection.");

        let (tx, mut rx) = mpsc::unbounded::<ClientCommand>();
        command_sender.set(Some(tx));

        let ws = match WebSocket::open("ws://127.0.0.1:3000/ws") {
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
                        if let Ok(event) = serde_json::from_str::<ServerEvent>(&text) {
                            tracing::debug!(?event, "Received event from server");
                            if let ServerEvent::StateChanged(new_state) = event {
                                let mut state_lock = install_state.write();
                                let mut data =
                                    state_lock.write().expect("BUG: state lock poisoned");
                                *data = Some(new_state);
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
