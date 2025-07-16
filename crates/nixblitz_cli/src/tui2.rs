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
use log::error;
use nixblitz_core::{
    OPTION_TITLES, SupportedApps,
    bool_data::BoolOptionChangeData,
    option_data::{GetOptionId, OptionData, OptionDataChangeNotification, OptionId},
    string_list_data::StringListOptionChangeData,
};
use nixblitz_system::project::Project;

use crate::tui2_components::{
    NavDirection, Popup, get_background_color, get_selected_char, navigate_selection,
    utils::format_bool_subtitle,
};
use crate::{errors::CliError, tui2_components::SelectList};

pub async fn start_tui2(
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

#[derive(Default, Props)]
struct AppListProps {
    has_focus: bool,
    app_list: &'static [&'static str],
    selected: usize,
}

#[component]
fn AppList(props: &mut AppListProps) -> impl Into<AnyElement<'static>> {
    let selected = props.selected;
    let items = props.app_list.iter().enumerate().map(|(i, app)| {
        let char = get_selected_char(i == selected);
        let background_color = get_background_color(i == selected);
        element! {
            View(background_color: background_color) {
                Text(content: format!("{} {}", char, app.to_string()))
            }
        }
    });

    element! {
        View(
            flex_direction: FlexDirection::Column,
            border_style: BorderStyle::Round,
        ) {
            #(items)
        }
    }
}

#[derive(Default, Props)]
struct OptionsListProps {
    has_focus: bool,
    on_edit_option: Handler<'static, OptionId>,
    options: Arc<Vec<OptionData>>,
}

const MAX_HEIGHT: u16 = 25;

#[component]
fn OptionList(props: &mut OptionsListProps, mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let (_, height) = hooks.use_terminal_size();
    let mut selected = hooks.use_state(|| 0);
    let options = &props.options;
    let num_opts = options.len();
    let height = height.min(MAX_HEIGHT);
    let max_num_list_items = (height as usize / 2) - 1; // minus one for the borders
    let mut offset = hooks.use_state(|| 0);

    hooks.use_terminal_events({
        let mut option_handler = props.on_edit_option.take();
        let options = options.clone();
        let f = props.has_focus;
        move |event| {
            if !f {
                return;
            }

            match event {
                TerminalEvent::Key(KeyEvent { code, kind, .. })
                    if kind != KeyEventKind::Release =>
                {
                    match code {
                        KeyCode::Char('j') => {
                            let res = navigate_selection(
                                NavDirection::Next,
                                selected.get(),
                                offset.get(),
                                num_opts,
                                max_num_list_items,
                            );
                            offset.set(res.offset);
                            selected.set(res.selected);
                        }
                        KeyCode::Char('k') => {
                            let res = navigate_selection(
                                NavDirection::Previous,
                                selected.get(),
                                offset.get(),
                                num_opts,
                                max_num_list_items,
                            );
                            offset.set(res.offset);
                            selected.set(res.selected);
                        }
                        KeyCode::Enter => {
                            let i = selected.get();
                            let o = options.get(i).unwrap().id();
                            option_handler(o.clone());
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    });

    let option_entries = options
        .iter()
        .enumerate()
        .skip(offset.get())
        .map(|(i, option)| {
            let char = get_selected_char(i == selected.get());
            let background_color = get_background_color(i == selected.get());
            let option = option.clone();
            let id = option.id();
            let title = OPTION_TITLES
                .get(id)
                .ok_or(CliError::OptionTitleRetrievalError(id.to_string()))
                .unwrap();

            match option {
                OptionData::Bool(b) => {
                    let subtitle = format_bool_subtitle(b.value());
                    element! {
                        View(
                            flex_direction: FlexDirection::Column,
                            background_color,
                        ) {
                            Text(content: format!("{} {}", char, title))
                            Text(content: format!("{} {}", char, subtitle))
                        }
                    }
                }
                _ => {
                    element! {
                        View(
                            flex_direction: FlexDirection::Column,
                            background_color,
                        ) {
                            Text(content: format!("{} {}", char, title))
                            Text(content: format!("{} {}", char, "subtitle"))
                        }
                    }
                }
            }
        })
        .take(max_num_list_items)
        .collect::<Vec<_>>();

    if selected.get() > num_opts {
        if num_opts <= max_num_list_items {
            offset.set(0);
        } else {
            offset.set(num_opts - max_num_list_items);
        }
        selected.set(num_opts - 1);
    }

    let content = format!("Offset: {}, Selected: {}", offset.get(), selected.get());
    element! {
        View(
            height: height + 2,
            flex_direction: FlexDirection::Column,
            border_style: BorderStyle::Round,
        ) {
            View (flex_direction: FlexDirection::Column) {
                #(option_entries)
                Text(content)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Focus {
    App,
    Option,
    Popup,
}

#[derive(Debug, Clone, PartialEq)]
enum PopupData {
    Help(String),
    Option(OptionData),
}

#[component]
fn App(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let mut system = hooks.use_context_mut::<SystemContext>();
    let project = hooks.use_context_mut::<Arc<Mutex<Project>>>();
    let mut show_help = hooks.use_state(|| false);
    let mut should_exit = hooks.use_state(|| false);
    let mut focus = hooks.use_state(|| Focus::Option);
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
                    KeyCode::Char('p') => {
                        let new_val = !show_popup.get();
                        show_popup.set(new_val);
                        if new_val {
                            focus.set(Focus::Popup);
                        } else {
                            focus.set(Focus::Option);
                        };
                    }
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
    let project_clone = project.clone();
    let on_edit_option = move |option_id: OptionId| {
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
                OptionData::StringList(_) => {
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
                _ => {}
            }
        } else {
            error!(
                "Option with id {:?} not found in current options state",
                option_id
            );
        }
    };

    let popup = if let Some(data) = popup_data.read().clone() {
        match data {
            PopupData::Help(_) => todo!(),
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
                    focus.set(Focus::Option);
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
            AppList(has_focus: focus.get() == Focus::App, app_list: SupportedApps::as_string_list(), selected: selected_app.get().as_index())
            OptionList(has_focus: focus.get() == Focus::Option, on_edit_option, options: options.read().clone())
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
            let on_close_requested = Arc::new(Mutex::new(Some(on_close_requested)));
            let on_selected = move |i: usize| {
                if let Some(selected) = s_for_closure.options().get(i) {
                    let mut project = project.lock().unwrap();
                    let res = project.on_option_changed(OptionDataChangeNotification::StringList(
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

                if let Some(cb) = on_close_requested.lock().unwrap().take() {
                    cb();
                }
            };

            Some(
                element! {
                    Popup(
                        has_focus: true,
                        title: "Choose wisely".to_string(),
                        children: vec![
                            element! {
                                SelectList(
                                    has_focus: true,
                                    options: s.options().clone(),
                                    on_selected,
                                )
                            }.into_any()
                        ]
                    )
                }
                .into_any(),
            )
        }
        _ => None,
    }
}
