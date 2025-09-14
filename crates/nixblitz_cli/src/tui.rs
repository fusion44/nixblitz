use std::{
    io, panic,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use crossterm::{
    cursor::{Hide, Show},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use error_stack::{Result, ResultExt};
use iocraft::prelude::*;
use log::{error, info, warn};
use nixblitz_core::{SystemClientCommand, SystemState};
use nixblitz_system::project::Project;
use tokio::sync::{oneshot, watch};

use crate::tui_components::utils::{get_focus_border_color, load_or_create_project};
use crate::{
    errors::CliError,
    tui_components::{EngineOffHelpPopup, LogViewer, Popup, Spinner, configurator::Configurator},
    tui_shared::{ConnectionStatus, Focus, PopupData, PopupDataState, ShowPopupState},
    tui_system_ws_utils::{TuiSystemEngineConnection, connect_and_manage, get_ws_url},
};

const MAX_HEIGHT: u16 = 24; // Maximum height of the TUI, will be +2 for borders
const MAX_TOTAL_WIDTH: u16 = 120; // Maximum width of AppList + OptionList

pub async fn start_tui_app(work_dir: PathBuf, create_project: &bool) -> Result<(), CliError> {
    let project = match load_or_create_project(&work_dir, *create_project).await? {
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
                App
            }
        }
        .render_loop()
        .await;
    })
    .await;

    restore_terminal();

    if let Err(e) = result {
        eprintln!("Render loop panicked: {:?}", e);
        return Err(CliError::GenericError("Render loop failed".to_string()).into());
    }

    Ok(())
}

#[component]
fn App(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let mut system = hooks.use_context_mut::<SystemContext>();

    // ui states
    let mut should_exit = hooks.use_state(|| false);

    // websocket states
    let (connection_status_tx, connection_status_rx) = watch::channel(ConnectionStatus::Connecting);
    let mut connection_status = hooks.use_state(|| ConnectionStatus::Connecting);
    let mut shutdown_tx = hooks.use_state(|| Option::<oneshot::Sender<()>>::None);
    let system_state = hooks.use_state(|| SystemState::Idle);
    let engine = hooks.use_state(|| Arc::new(Mutex::new(TuiSystemEngineConnection::new())));

    // popup states
    let show_popup: ShowPopupState = Arc::new(Mutex::new(hooks.use_state(|| false)));
    let popup_data: PopupDataState = Arc::new(Mutex::new(hooks.use_state(|| None)));
    let switch_logs = hooks.use_state(Vec::new);

    hooks.use_future({
        let mut rx = connection_status_rx.clone();
        async move {
            while let Ok(()) = rx.changed().await {
                let status = *rx.borrow();
                connection_status.set(status);
            }
        }
    });

    hooks.use_future({
        let e = engine.read().clone();
        let s_logs = Arc::new(Mutex::new(switch_logs));
        let s_popup = show_popup.clone();
        let p_data = popup_data.clone();
        let connection_status_tx = connection_status_tx.clone();

        async move {
            let (tx, rx) = oneshot::channel();
            shutdown_tx.set(Some(tx));

            connect_and_manage(
                e,
                &get_ws_url(),
                system_state,
                s_logs,
                s_popup,
                p_data,
                rx,
                connection_status_tx,
            )
            .await;
        }
    });

    hooks.use_terminal_events({
        let engine = engine.read().clone();
        let show_popup_clone = show_popup.clone();
        let popup_data_clone = popup_data.clone();
        move |event| {
            if show_popup_clone.lock().unwrap().get() {
                return;
            }
            if let TerminalEvent::Key(KeyEvent {
                code,
                kind: KeyEventKind::Press,
                modifiers,
                ..
            }) = event
            {
                match code {
                    KeyCode::Char('q') => {
                        if let Some(tx) = shutdown_tx.write().take() {
                            let _ = tx.send(());
                        }
                        should_exit.set(true);
                    }
                    KeyCode::Char('s') if modifiers == KeyModifiers::CONTROL => {
                        if !show_popup_clone.lock().unwrap().get()
                            && connection_status.get() == ConnectionStatus::Connected
                        {
                            engine
                                .lock()
                                .unwrap()
                                .send_command(SystemClientCommand::SwitchConfig);
                        } else if !show_popup_clone.lock().unwrap().get()
                            && connection_status.get() == ConnectionStatus::Disconnected
                        {
                            show_popup_clone.lock().unwrap().set(true);
                            popup_data_clone
                                .lock()
                                .unwrap()
                                .set(Some(PopupData::EngineOffHelp));
                        }
                    }
                    _ => {}
                }
            }
        }
    });

    if should_exit.get() {
        system.exit();
    }

    let (width, height) = hooks.use_terminal_size();

    // When we show a popup, we need to tell our child elements that
    // they are not focused
    let focused = !show_popup.lock().unwrap().get();
    let popup = if let Some(data) = popup_data.lock().unwrap().read().clone() {
        let popup_data_clone = popup_data.clone();
        let show_popup_clone = show_popup.clone();
        match data {
            PopupData::Update => {
                let switch_logs = switch_logs.read().clone();

                build_update_popup(switch_logs, move || {}, focused)
            }
            PopupData::EngineOffHelp => build_engine_off_help_popup(
                move || {
                    popup_data_clone.lock().unwrap().set(None);
                    show_popup_clone.lock().unwrap().set(false);
                },
                focused,
            ),
            _ => None,
        }
    } else {
        None
    };

    let status_bar = match connection_status.get() {
        ConnectionStatus::Connecting => Some(element! {
            MixedText(
                align: TextAlign::Center,
                contents: vec![
                    MixedTextContent::new("Connecting to system engine..."),
                ]
            )
        }),
        ConnectionStatus::Connected => Some(element! {
            MixedText(
                align: TextAlign::Center,
                contents: vec![
                    MixedTextContent::new(" <"),
                    MixedTextContent::new("CTRL + s").color(Color::Green),
                    MixedTextContent::new("> Switch Config"),
                    MixedTextContent::new(" <"),
                    MixedTextContent::new("q").color(Color::Green),
                    MixedTextContent::new("> Quit"),
                ]
            )
        }),
        ConnectionStatus::Disconnected => Some(element! {
            MixedText(
                align: TextAlign::Center,
                contents: vec![
                    MixedTextContent::new("Operating in non-system engine mode. Press '"),
                    MixedTextContent::new("q").color(Color::Green),
                    MixedTextContent::new("' to quit."),
                ]
            )
        }),
    };

    element! {
        View (
            width,
            height,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
        ) {
            View (
                background_color: Color::Reset,
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                height: MAX_HEIGHT,
            ) {
                Configurator(on_submit: None, has_focus: Some(focused))
                #(popup)
            }
            View(
                height: 3,
                width: MAX_TOTAL_WIDTH.min(width),
                background_color: Color::Reset,
                border_style: BorderStyle::Round,
                border_color: get_focus_border_color(false),
                ) {
                    #(status_bar)
            }
        }
    }
}

fn build_engine_off_help_popup<F>(
    mut on_close_requested: F,
    focused: bool,
) -> Option<AnyElement<'static>>
where
    F: FnMut() + Send + 'static + Sync,
{
    Some(
        element! {
            EngineOffHelpPopup(
                on_confirm: move |_| { on_close_requested() },
                has_focus: focused,
            )
        }
        .into_any(),
    )
}

fn build_update_popup<F>(
    logs: Vec<String>,
    _on_close_requested: F,
    focused: bool,
) -> Option<AnyElement<'static>>
where
    F: FnOnce() + Send + 'static,
{
    Some(
        element! {
            Popup(
                has_focus: focused,
                title: " Switching config…".to_string(),
                spinner: Some(element! {
                    Spinner()
                }.into_any()),
                children: vec![
                    element! {
                        View(
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center
                        ) {
                            LogViewer(logs, max_height: 25, width: 50)
                        }
                    }.into_any()
                ]
            )
        }
        .into_any(),
    )
}
