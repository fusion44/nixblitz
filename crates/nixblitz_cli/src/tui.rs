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
use log::{error, warn};
use nixblitz_core::{
    OPTION_TITLES, SupportedApps, SystemClientCommand, SystemState,
    bool_data::BoolOptionChangeData,
    manual_string_list_data::ManualStringListOptionChangeData,
    net_address_data::NetAddressOptionChangeData,
    number_data::NumberOptionChangeData,
    option_data::{GetOptionId, OptionData, OptionDataChangeNotification},
    password_data::PasswordOptionChangeData,
    port_data::PortOptionChangeData,
    string_list_data::StringListOptionChangeData,
    text_edit_data::TextOptionChangeData,
};
use nixblitz_system::project::Project;
use tokio::sync::{oneshot, watch};

use crate::{
    errors::CliError,
    tui_components::{EngineOffHelpPopup, app_list::AppList},
    tui_shared::{Focus, FocusState},
    tui_system_ws_utils::{connect_and_manage, get_ws_url},
};
use crate::{
    tui_components::{
        LogViewer, NetAddressPopup, NetAddressPopupResult, NumberPopup, NumberPopupResult,
        PasswordInputMode, PasswordInputPopup, PasswordInputResult, Popup, SelectableList,
        SelectableListData, SelectionValue, Spinner, TextInputPopup, TextInputPopupResult,
        app_option_list::AppOptionList,
        utils::{SelectableItem, get_focus_border_color, load_or_create_project},
    },
    tui_shared::{ConnectionStatus, PopupData, PopupDataState, ShowPopupState, SwitchLogsState},
    tui_system_ws_utils::TuiSystemEngineConnection,
};

const MAX_HEIGHT: u16 = 24; // Maximum height of the TUI, will be +2 for borders
const MAX_TOTAL_WIDTH: u16 = 120; // Maximum width of AppList + OptionList
const APP_LIST_WIDTH: u16 = 20;
const MIN_OPTION_WIDTH: u16 = 40;
const PADDING: u16 = 2;

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

fn reset_popup(data: PopupDataState, show: ShowPopupState, focus: FocusState) {
    data.lock()
        .expect("BUG: popup_data lock poisoned")
        .set(None);
    show.lock()
        .expect("BUG: show_popup lock poisoned")
        .set(false);
    focus
        .lock()
        .expect("BUG: focus lock poisoned")
        .set(Focus::OptionList);
}

#[component]
fn App(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let mut system = hooks.use_context_mut::<SystemContext>();
    let project = hooks.use_context_mut::<Arc<Mutex<Project>>>();

    // ui states
    let focus: FocusState = Arc::new(Mutex::new(hooks.use_state(|| Focus::OptionList)));
    let mut selected_app = hooks.use_state(|| SupportedApps::NixOS);
    let mut should_exit = hooks.use_state(|| false);
    let mut show_help = hooks.use_state(|| false);

    // websocket states
    let (connection_status_tx, connection_status_rx) = watch::channel(ConnectionStatus::Connecting);
    let mut connection_status = hooks.use_state(|| ConnectionStatus::Connecting);
    let mut shutdown_tx = hooks.use_state(|| Option::<oneshot::Sender<()>>::None);
    let system_state = hooks.use_state(|| SystemState::Idle);
    let engine = hooks.use_state(|| Arc::new(Mutex::new(TuiSystemEngineConnection::new())));

    // popup states
    let show_popup: ShowPopupState = Arc::new(Mutex::new(hooks.use_state(|| false)));
    let popup_data: PopupDataState = Arc::new(Mutex::new(hooks.use_state(|| None)));
    let switch_logs: SwitchLogsState = Arc::new(Mutex::new(hooks.use_state(Vec::new)));

    let mut options: State<Arc<Vec<OptionData>>> = hooks.use_state(|| {
        project
            .lock()
            .unwrap()
            .get_app_options()
            .unwrap_or_default()
    });

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
        let s_logs = switch_logs.clone();
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

    let mut on_app_selected = {
        let project = project.clone();
        let focus = focus.clone();
        move |reverse: bool| {
            if focus.lock().expect("BUG: focus lock poisoned").get() != Focus::OptionList {
                return;
            }

            let next_app = if reverse {
                selected_app.get().previous()
            } else {
                selected_app.get().next()
            };
            project.lock().unwrap().set_selected_app(next_app);
            selected_app.set(next_app);
            options.set(
                project
                    .lock()
                    .unwrap()
                    .get_app_options()
                    .unwrap_or_default(),
            );
        }
    };

    hooks.use_terminal_events({
        let engine = engine.read().clone();
        let show_popup_clone = show_popup.clone();
        let popup_data_clone = popup_data.clone();
        let focus_clone = focus.clone();
        move |event| {
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
                    KeyCode::Tab => on_app_selected(false),
                    KeyCode::BackTab => on_app_selected(true),
                    KeyCode::Char('?') => show_help.set(!show_help.get()),
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
                            show_popup_clone
                                .lock()
                                .expect("BUG: show_popup lock poisoned")
                                .set(true);
                            popup_data_clone
                                .lock()
                                .expect("BUG: popup_data lock poisoned")
                                .set(Some(PopupData::EngineOffHelp));
                            focus_clone
                                .lock()
                                .expect("BUG: focus lock poisoned")
                                .set(Focus::Popup);
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

    let total_available_width = width.min(MAX_TOTAL_WIDTH);
    let option_list_width = total_available_width
        .saturating_sub(APP_LIST_WIDTH)
        .saturating_sub(PADDING)
        .max(MIN_OPTION_WIDTH);

    let project_clone = project.clone();
    let show_popup_clone = show_popup.clone();
    let popup_data_clone = popup_data.clone();
    let focus_clone = focus.clone();

    let on_edit_option = move |selection: Option<SelectionValue>| {
        if let Some(SelectionValue::OptionId(option_id)) = selection {
            let current_options = options.read().clone();
            let option_data = current_options
                .iter()
                .find(|o| *o.id() == option_id)
                .cloned();

            if let Some(option_data) = option_data {
                let project_clone = project_clone.clone();
                let show_popup_clone = show_popup_clone.clone();
                let popup_data_clone = popup_data_clone.clone();
                let focus_clone = focus_clone.clone();

                match option_data {
                    OptionData::Bool(b) => {
                        let mut p = project_clone.lock().unwrap();
                        let change_notification = OptionDataChangeNotification::Bool(
                            BoolOptionChangeData::new(option_id.clone(), !b.value()),
                        );

                        let res = p.on_option_changed(change_notification);
                        if let Err(e) = res {
                            error!("Error setting option: {:?}", e);
                        } else {
                            let new_options = p.get_app_options().unwrap_or_default();
                            drop(p);
                            options.set(new_options);
                        }
                    }
                    OptionData::StringList(_)
                    | OptionData::ManualStringList(_)
                    | OptionData::TextEdit(_)
                    | OptionData::PasswordEdit(_)
                    | OptionData::NetAddress(_)
                    | OptionData::Port(_)
                    | OptionData::NumberEdit(_) => {
                        if show_popup_clone
                            .lock()
                            .expect("BUG: show_popup lock poisoned")
                            .get()
                        {
                            error!(
                                "Trying to open a string list popup while another popup is already open"
                            );
                            return;
                        }

                        popup_data_clone
                            .lock()
                            .expect("BUG: popup_data lock poisoned")
                            .set(Some(PopupData::Option(option_data)));
                        show_popup_clone
                            .lock()
                            .expect("BUG: show_popup lock poisoned")
                            .set(true);
                        focus_clone
                            .lock()
                            .expect("BUG: show_popup lock poisoned")
                            .set(Focus::Popup);
                    }
                    _ => {
                        println!("Option {:?} not handled, yet", option_data);
                    }
                }
            } else {
                error!(
                    "Option with id {:?} not found in current options state",
                    option_id
                );
            }
        }
    };

    let popup = if let Some(data) = popup_data
        .lock()
        .expect("BUG: popup_data lock poisoned")
        .read()
        .clone()
    {
        let popup_data_clone = popup_data.clone();
        let show_popup_clone = show_popup.clone();
        let focus_clone = focus.clone();
        match data {
            PopupData::Option(option_data) => {
                let project_for_popup = project.clone();
                build_option_popup(project.clone(), option_data, move || {
                    let new_options = project_for_popup
                        .lock()
                        .unwrap()
                        .get_app_options()
                        .unwrap_or_default();
                    options.set(new_options);
                    reset_popup(popup_data_clone, show_popup_clone, focus_clone);
                })
            }
            PopupData::Update => {
                let switch_logs = switch_logs
                    .lock()
                    .expect("BUG: switch_logs lock poisoned")
                    .read()
                    .clone();

                build_update_popup(switch_logs, move || {})
            }
            PopupData::EngineOffHelp => build_engine_off_help_popup(move || {
                reset_popup(
                    popup_data_clone.clone(),
                    show_popup_clone.clone(),
                    focus_clone.clone(),
                );
            }),
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

    let focus = focus.lock().expect("BUG: focus lock poisoned").get();
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
            ) {
                AppList (
                    has_focus: focus == Focus::AppList,
                    app_list: SupportedApps::as_string_list(),
                    selected_item: selected_app.get().as_index(),
                    width: APP_LIST_WIDTH,
                    height: Some(MAX_HEIGHT),
                )
                AppOptionList (
                        height: MAX_HEIGHT,
                        width: option_list_width,
                        has_focus: focus == Focus::OptionList,
                        on_edit_option,
                        options: options.read().clone(),
                )
                #(popup)
            }
            View(
                height: 3,
                width: option_list_width + APP_LIST_WIDTH,
                background_color: Color::Reset,
                border_style: BorderStyle::Round,
                border_color: get_focus_border_color(false),
                ) {
                    #(status_bar)
            }
        }
    }
}

fn build_engine_off_help_popup<F>(mut on_close_requested: F) -> Option<AnyElement<'static>>
where
    F: FnMut() + Send + 'static + Sync,
{
    Some(element! { EngineOffHelpPopup(on_confirm: move |_| { on_close_requested() }) }.into_any())
}

fn build_update_popup<F>(logs: Vec<String>, _on_close_requested: F) -> Option<AnyElement<'static>>
where
    F: FnOnce() + Send + 'static,
{
    Some(
        element! {
            Popup(
                has_focus: true,
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

fn build_option_popup<F>(
    project: Arc<Mutex<Project>>,
    data: OptionData,
    on_close_requested: F,
) -> Option<AnyElement<'static>>
where
    F: FnOnce() + Send + 'static,
{
    match data {
        OptionData::StringList(s) => {
            let s_for_closure = s.clone();
            let title = OPTION_TITLES.get(s.id()).map_or("title_not_found", |v| v);

            // Not, this callbacks only function is to notify the caller
            // that the popup can be closed. No value should be passed back.
            // This should probably be refactored such that the popup closes itself
            let on_close_requested = Arc::new(Mutex::new(Some(on_close_requested)));
            let on_selected = move |selection: Option<SelectionValue>| {
                if let Some(SelectionValue::Index(i)) = selection {
                    if let Some(selected) = s_for_closure.options().get(i) {
                        let mut project = project.lock().unwrap();
                        let res =
                            project.on_option_changed(OptionDataChangeNotification::StringList(
                                StringListOptionChangeData::new(
                                    s_for_closure.id().clone(),
                                    selected.value.clone(),
                                ),
                            ));
                        if let Err(e) = res {
                            error!("Error setting option: {:?}", e);
                        }
                    } else {
                        error!("Option index out of bounds");
                    }
                }

                if let Some(cb) = on_close_requested.lock().unwrap().take() {
                    cb();
                }
            };

            let options = s.options().clone();
            let num_options = options.len();
            let item_height = options.first().map_or(1, |item| item.item_height());
            let max_num_list_items = (MAX_HEIGHT / item_height) as usize;
            let height = if num_options > max_num_list_items {
                Some(MAX_HEIGHT - 4)
            } else {
                None
            };
            Some(
                element! {
                    Popup(
                        has_focus: true,
                        title: title.to_string(),
                        children: vec![
                            element! {
                                SelectableList(
                                    height,
                                    width: Some(40),
                                    has_focus: true,
                                    on_selected,
                                    data: SelectableListData::StringListItems(options),
                                )
                            }.into_any()
                        ]
                    )
                }
                .into_any(),
            )
        }
        OptionData::ManualStringList(string_list_data) => {
            let id = string_list_data.id().clone();
            let title = OPTION_TITLES.get(&id).map_or("", |v| v);
            let on_close_requested = Arc::new(Mutex::new(Some(on_close_requested)));
            let on_submit = move |result| {
                match result {
                    TextInputPopupResult::Accepted(value) => {
                        let mut lines: Vec<String> =
                            Vec::with_capacity(value.matches('\n').count());
                        for line in value.lines() {
                            lines.push(line.to_string());
                        }
                        let res = project.lock().unwrap().on_option_changed(
                            OptionDataChangeNotification::ManualStringList(
                                ManualStringListOptionChangeData::new(id.clone(), lines),
                            ),
                        );
                        if let Err(e) = res {
                            error!("Error setting option: {:?}", e);
                        }
                    }
                    TextInputPopupResult::Cancelled => {}
                };
                if let Some(cb) = on_close_requested.lock().unwrap().take() {
                    cb();
                }
            };

            let height = if string_list_data.max_lines() > MAX_HEIGHT - 4 {
                Some(MAX_HEIGHT - 4)
            } else {
                None
            };
            let text_data = string_list_data.value().join("\n");
            Some(
                element! {
                    TextInputPopup(
                        height,
                        title,
                        text: text_data,
                        max_lines: 999 as u16,
                        on_submit,
                    )
                }
                .into_any(),
            )
        }
        OptionData::TextEdit(text_data) => {
            let id = text_data.id().clone();
            let title = OPTION_TITLES.get(&id).map_or("", |v| v);
            let on_close_requested = Arc::new(Mutex::new(Some(on_close_requested)));
            let on_submit = move |result| {
                match result {
                    TextInputPopupResult::Accepted(value) => {
                        let res = project.lock().unwrap().on_option_changed(
                            OptionDataChangeNotification::TextEdit(TextOptionChangeData::new(
                                id.clone(),
                                value,
                            )),
                        );
                        if let Err(e) = res {
                            error!("Error setting option: {:?}", e);
                        }
                    }
                    TextInputPopupResult::Cancelled => {}
                };
                if let Some(cb) = on_close_requested.lock().unwrap().take() {
                    cb();
                }
            };

            let height = if text_data.max_lines() > MAX_HEIGHT - 4 {
                Some(MAX_HEIGHT - 4)
            } else {
                None
            };
            Some(
                element! {
                    TextInputPopup(
                        height,
                        title,
                        text: text_data.value().clone(),
                        max_lines: text_data.max_lines(),
                        on_submit,
                    )
                }
                .into_any(),
            )
        }
        OptionData::PasswordEdit(password_data) => {
            let id = password_data.id().clone();
            let title = OPTION_TITLES.get(&id).map_or("", |v| v);
            let on_close_requested = Arc::new(Mutex::new(Some(on_close_requested)));

            let on_submit = move |result| match result {
                PasswordInputResult::Accepted(value) => {
                    let res =
                        project.lock().unwrap().on_option_changed(
                            OptionDataChangeNotification::PasswordEdit(
                                PasswordOptionChangeData::new(id.clone(), value, None),
                            ),
                        );
                    if let Err(e) = res {
                        error!("Error setting option: {:?}", e);
                    }
                    if let Some(cb) = on_close_requested.lock().unwrap().take() {
                        cb();
                    }
                }
                PasswordInputResult::Cancelled => {
                    if let Some(cb) = on_close_requested.lock().unwrap().take() {
                        cb();
                    }
                }
            };

            Some(
                element! {
                    PasswordInputPopup(
                        title,
                        mode: PasswordInputMode::SetNewPassword,
                        on_submit,
                    )
                }
                .into_any(),
            )
        }
        OptionData::NetAddress(net_address_data) => {
            let id = net_address_data.id().clone();
            let title = OPTION_TITLES.get(&id).map_or("", |v| v);
            let on_close_requested = Arc::new(Mutex::new(Some(on_close_requested)));
            let on_submit = move |result| {
                match result {
                    NetAddressPopupResult::Accepted(value) => {
                        let res = project.lock().unwrap().on_option_changed(
                            OptionDataChangeNotification::NetAddress(
                                NetAddressOptionChangeData::new(id.clone(), value),
                            ),
                        );
                        if let Err(e) = res {
                            error!("Error setting option: {:?}", e);
                        }
                    }
                    NetAddressPopupResult::Cancelled => {}
                };
                if let Some(cb) = on_close_requested.lock().unwrap().take() {
                    cb();
                }
            };

            Some(
                element! {
                    NetAddressPopup(
                        title,
                        text: net_address_data.value().map_or(
                            String::new(), |v| v.to_string()),
                        on_submit,
                    )
                }
                .into_any(),
            )
        }
        OptionData::Port(port_data) => {
            let id = port_data.id().clone();
            let title = OPTION_TITLES.get(&id).map_or("", |v| v);
            let on_close_requested = Arc::new(Mutex::new(Some(on_close_requested)));
            let on_submit = move |result| {
                match result {
                    NumberPopupResult::Accepted(value) => {
                        let res = project.lock().unwrap().on_option_changed(
                            OptionDataChangeNotification::Port(PortOptionChangeData::new(
                                id.clone(),
                                value,
                            )),
                        );
                        if let Err(e) = res {
                            error!("Error setting option: {:?}", e);
                        }
                    }
                    NumberPopupResult::Cancelled => {}
                };
                if let Some(cb) = on_close_requested.lock().unwrap().take() {
                    cb();
                }
            };

            Some(
                element! {
                    NumberPopup(
                        title,
                        value: port_data.value().clone(),
                        on_submit,
                    )
                }
                .into_any(),
            )
        }
        OptionData::NumberEdit(number_data) => {
            let id = number_data.id().clone();
            let title = OPTION_TITLES.get(&id).map_or("", |v| v);
            let on_close_requested = Arc::new(Mutex::new(Some(on_close_requested)));
            let on_submit = move |result| {
                match result {
                    NumberPopupResult::Accepted(value) => {
                        let res = project.lock().unwrap().on_option_changed(
                            OptionDataChangeNotification::Number(NumberOptionChangeData::new(
                                id.clone(),
                                value,
                            )),
                        );
                        if let Err(e) = res {
                            error!("Error setting option: {:?}", e);
                        }
                    }
                    NumberPopupResult::Cancelled => {}
                };
                if let Some(cb) = on_close_requested.lock().unwrap().take() {
                    cb();
                }
            };

            Some(
                element! {
                    NumberPopup(
                        title,
                        value: number_data.value().clone(),
                        on_submit,
                    )
                }
                .into_any(),
            )
        }
        _ => {
            // Note, bool won't be handled here as it doesn't require a popup
            // and is handled inline
            warn!("Option {} not handled, yet", data);
            None
        }
    }
}
