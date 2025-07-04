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
};
use nixblitz_system::project::Project;

use crate::errors::CliError;

enum NavDirection {
    Previous,
    Next,
}

pub fn get_focus_border_color(has_focus: bool) -> Color {
    if has_focus { Color::Green } else { Color::Blue }
}

pub fn get_selected_char(selected: bool) -> char {
    if selected { '>' } else { ' ' }
}

pub fn get_background_color(selected: bool) -> Color {
    if selected {
        Color::DarkBlue
    } else {
        Color::Reset
    }
}

pub fn format_bool_subtitle(value: bool) -> String {
    if value {
        return "✓ (true)".to_string();
    }

    "✗ (false)".to_string()
}

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
    app: SupportedApps,
}

const MAX_HEIGHT: u16 = 20;

struct NavSelectionResult {
    selected: usize,
    offset: usize,
}

impl NavSelectionResult {
    fn new(selected: usize, offset: usize) -> Self {
        Self { selected, offset }
    }
}

fn navigate_selection(
    direction: NavDirection,
    selected: usize,
    offset: usize,
    options_len: usize,
    max_num_items: usize,
) -> NavSelectionResult {
    if options_len == 0 {
        return NavSelectionResult::new(0, 0);
    }

    match direction {
        NavDirection::Previous => {
            if selected == 0 {
                NavSelectionResult::new(0, 0)
            } else if offset > 0 && offset == selected {
                NavSelectionResult::new(selected - 1, offset - 1)
            } else {
                NavSelectionResult::new(selected - 1, offset)
            }
        }
        NavDirection::Next => {
            if selected == options_len - 1 {
                NavSelectionResult::new(selected, offset)
            } else if offset + max_num_items - 1 == selected {
                NavSelectionResult::new(selected + 1, offset + 1)
            } else {
                NavSelectionResult::new(selected + 1, offset)
            }
        }
    }
}

#[component]
fn OptionList(props: &mut OptionsListProps, mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let (_, height) = hooks.use_terminal_size();
    let mut selected = hooks.use_state(|| 0);
    let project = hooks.use_context_mut::<Arc<Mutex<Project>>>();
    let options = project.lock().unwrap().get_app_options().unwrap().clone();
    let num_opts = options.len();
    let height = height.min(MAX_HEIGHT);
    let max_num_list_items = (height as usize / 2) - 1; // minus one for the borders
    let mut offset = hooks.use_state(|| 0);

    hooks.use_terminal_events({
        let mut option_handler = props.on_edit_option.take();
        let options = options.clone();
        move |event| match event {
            TerminalEvent::Key(KeyEvent { code, kind, .. }) if kind != KeyEventKind::Release => {
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
}

#[component]
fn App(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let mut system = hooks.use_context_mut::<SystemContext>();
    let project = hooks.use_context_mut::<Arc<Mutex<Project>>>();
    let mut show_help = hooks.use_state(|| false);
    let mut should_exit = hooks.use_state(|| false);
    let focus = hooks.use_state(|| Focus::App);
    let mut selected_app = hooks.use_state(|| SupportedApps::NixOS);

    let project_clone = project.clone();
    let mut on_app_selected = move |reverse: bool| {
        let next_app = match reverse {
            true => selected_app.get().previous(),
            false => selected_app.get().next(),
        };
        project_clone.lock().unwrap().set_selected_app(next_app);
        selected_app.set(next_app);
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
    let help = if show_help.get() {
        let w = if width > 100 { 100 } else { width };
        let h = if height > 20 { 20 } else { height };

        Some(element! { View(
                width: w,
                height: h,
                background_color: Color::Reset,
                border_style: BorderStyle::Round,
                position: Position::Absolute,
            ) {
                Text(content: "Help")
            }
        })
    } else {
        None
    };

    let project_clone = project.clone();
    let on_edit_option = move |option_id: OptionId| {
        // dang, craaaaaaaaazyyy
        let binding = project_clone
            .lock()
            .unwrap()
            .get_app_options()
            .unwrap()
            .clone();
        let binding = binding
            .iter()
            .filter(|o| *o.id() == option_id)
            .collect::<Vec<_>>();
        let option_data = binding.first().unwrap();

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
                }
            }
            _ => {}
        }
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
            OptionList(has_focus: focus.get() == Focus::Option, on_edit_option, app:selected_app.get())
            #(help)
        }
    }
}
