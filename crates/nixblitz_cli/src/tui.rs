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
    OPTION_TITLES, SupportedApps,
    bool_data::BoolOptionChangeData,
    net_address_data::NetAddressOptionChangeData,
    number_data::NumberOptionChangeData,
    option_data::{GetOptionId, OptionData, OptionDataChangeNotification},
    password_data::PasswordOptionChangeData,
    port_data::PortOptionChangeData,
    string_list_data::StringListOptionChangeData,
    text_edit_data::TextOptionChangeData,
};
use nixblitz_system::project::Project;

use crate::tui_components::{
    NetAddressPopup, NetAddressPopupResult, NumberPopup, NumberPopupResult, PasswordInputMode,
    PasswordInputPopup, PasswordInputResult, Popup, SelectableList, SelectableListData,
    SelectionValue, TextInputPopup, TextInputPopupResult,
    utils::{SelectableItem, get_focus_border_color},
};
use crate::{errors::CliError, tui_components::app_list::AppList};

const MAX_HEIGHT: u16 = 24; // Maximum height of the TUI, will be +2 for borders
const MAX_TOTAL_WIDTH: u16 = 120; // Maximum width of AppList + OptionList
const APP_LIST_WIDTH: u16 = 20;
const MIN_OPTION_WIDTH: u16 = 40;
const PADDING: u16 = 2;

pub async fn start_tui_app(
    _tick_rate: f64,
    _frame_rate: f64,
    work_dir: PathBuf,
) -> Result<(), CliError> {
    let project = Arc::new(Mutex::new(Project::load(work_dir.clone()).change_context(
        CliError::GenericError(format!("Unable to load project in dir {:?}", work_dir)),
    )?));

    fn restore_terminal() {
        if let Err(e) = disable_raw_mode() {
            eprintln!("Failed to disable raw mode: {}", e);
        }
        if let Err(e) = execute!(io::stdout(), LeaveAlternateScreen, Show) {
            eprintln!("Failed to leave alternate screen: {}", e);
        }
    }

    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        restore_terminal();
        original_hook(panic_info);
    }));

    let mut stdout = io::stdout();

    enable_raw_mode().change_context(CliError::UnableToStartTui)?;
    execute!(stdout, EnterAlternateScreen, Hide).change_context(CliError::UnableToStartTui)?;

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

#[derive(Debug, Clone, strum::Display, Copy, PartialEq, Eq)]
enum Focus {
    AppList,
    OptionList,
    Popup,
}

#[derive(Debug, Clone, strum::Display, PartialEq)]
enum PopupData {
    Option(OptionData),
}

#[component]
fn App(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let mut system = hooks.use_context_mut::<SystemContext>();
    let project = hooks.use_context_mut::<Arc<Mutex<Project>>>();
    let mut show_help = hooks.use_state(|| false);
    let mut should_exit = hooks.use_state(|| false);
    let mut focus = hooks.use_state(|| Focus::OptionList);
    let mut selected_app = hooks.use_state(|| SupportedApps::NixOS);
    let mut show_popup = hooks.use_state(|| false);
    let mut popup_data: State<Option<PopupData>> = hooks.use_state(|| None);

    let mut options: State<Arc<Vec<OptionData>>> = hooks.use_state(|| {
        project
            .lock()
            .unwrap()
            .get_app_options()
            .unwrap_or_default()
    });

    let project_clone = project.clone();
    let mut on_app_selected = move |reverse: bool| {
        if focus.get() != Focus::OptionList {
            // We only want to change the app if the focus is on the option list
            return;
        }

        let next_app = match reverse {
            true => selected_app.get().previous(),
            false => selected_app.get().next(),
        };
        project_clone.lock().unwrap().set_selected_app(next_app);
        selected_app.set(next_app);

        let new_options = project_clone
            .lock()
            .unwrap()
            .get_app_options()
            .unwrap_or_default();
        options.set(new_options);
    };

    hooks.use_terminal_events({
        move |event| match event {
            TerminalEvent::Key(KeyEvent { code, kind, .. }) if kind != KeyEventKind::Release => {
                match code {
                    KeyCode::Tab => on_app_selected(false),
                    KeyCode::BackTab => on_app_selected(true),
                    KeyCode::Char('q') => should_exit.set(true),
                    KeyCode::Char('?') => {
                        if !show_help.get() {
                            show_help.set(true)
                        } else {
                            show_help.set(false)
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
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
    let on_edit_option = move |selection: Option<SelectionValue>| {
        if let Some(SelectionValue::OptionId(option_id)) = selection {
            let current_options = options.read().clone();
            let option_data = current_options
                .iter()
                .find(|o| *o.id() == option_id)
                .cloned();

            if let Some(option_data) = option_data {
                let project_clone = project_clone.clone();

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
                    | OptionData::TextEdit(_)
                    | OptionData::PasswordEdit(_)
                    | OptionData::NetAddress(_)
                    | OptionData::Port(_)
                    | OptionData::NumberEdit(_) => {
                        if show_popup.get() {
                            error!(
                                "Trying to open a string list popup while another popup is already open"
                            );
                            return;
                        }

                        popup_data.set(Some(PopupData::Option(option_data)));
                        show_popup.set(true);
                        focus.set(Focus::Popup);
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

    let popup = if let Some(data) = popup_data.read().clone() {
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

                    popup_data.set(None);
                    show_popup.set(false);
                    focus.set(Focus::OptionList);
                })
            }
        }
    } else {
        None
    };

    element! {
        View(
            width,
            height,
            background_color: Color::Reset,
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
        ) {
            AppList(
                has_focus: focus.get() == Focus::AppList,
                app_list: SupportedApps::as_string_list(),
                selected_item: selected_app.get().as_index(),
                width: APP_LIST_WIDTH,
                height: Some(MAX_HEIGHT),
            )
            View(
                width: option_list_width,
                height: MAX_HEIGHT,
                border_style: BorderStyle::Round,
                border_color: get_focus_border_color(focus.get() == Focus::OptionList),
                justify_content: JustifyContent::Stretch,
            ) {
                SelectableList(
                    height: MAX_HEIGHT - 2, // -2 for borders
                    width: option_list_width - 2,
                    has_focus: focus.get() == Focus::OptionList,
                    on_selected: on_edit_option,
                    data: SelectableListData::Options(options.read().clone()),
                )
            }
            #(popup)
        }
    }
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
            warn!("Option {} not handled, yet", data);
            None
        }
    }
}
