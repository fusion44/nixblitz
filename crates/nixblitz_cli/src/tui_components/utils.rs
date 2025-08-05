use std::path::Path;

use error_stack::{Result, ResultExt};
use iocraft::{AnyElement, Color, ElementExt, element};
use log::info;
use nixblitz_system::{
    project::Project,
    utils::{init_default_project, safety_checks},
};

use crate::{errors::CliError, tui_components::ConfirmInputInline};

pub enum NavDirection {
    Previous,
    Next,
}

pub fn get_focus_border_color(has_focus: bool) -> Color {
    if has_focus { Color::Green } else { Color::Grey }
}

pub fn get_selected_item_color(selected: bool, component_focused: bool) -> Color {
    if selected && component_focused {
        Color::Blue
    } else if selected && !component_focused {
        Color::DarkGrey
    } else {
        get_background_color()
    }
}

pub fn get_selected_char(selected: bool) -> &'static char {
    if selected { &'>' } else { &' ' }
}

pub fn get_background_color() -> Color {
    Color::Reset
}

pub fn get_text_input_color(focused: bool, error: bool) -> Color {
    match (focused, error) {
        (true, true) => Color::DarkMagenta,
        (false, true) => Color::Magenta,
        (true, false) => Color::DarkGrey,
        (false, false) => Color::Grey,
    }
}

pub fn format_bool_subtitle(value: bool) -> String {
    if value {
        return "✓ (true)".to_string();
    }

    "✗ (false)".to_string()
}

pub struct NavSelectionResult {
    pub selected: usize,
    pub offset: usize,
}

impl NavSelectionResult {
    fn new(selected: usize, offset: usize) -> Self {
        Self { selected, offset }
    }
}

pub fn navigate_selection(
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
            if selected == 0 && options_len <= max_num_items {
                NavSelectionResult::new(options_len - 1, 0)
            } else if selected == 0 && options_len > max_num_items {
                NavSelectionResult::new(options_len - max_num_items, options_len - max_num_items)
            } else if offset > 0 && offset == selected {
                NavSelectionResult::new(selected - 1, offset - 1)
            } else {
                NavSelectionResult::new(selected - 1, offset)
            }
        }
        NavDirection::Next => {
            if selected == options_len - 1 {
                NavSelectionResult::new(0, 0)
            } else if offset + max_num_items - 1 == selected {
                NavSelectionResult::new(selected + 1, offset + 1)
            } else {
                NavSelectionResult::new(selected + 1, offset)
            }
        }
    }
}

/// Generic item trait that can be implemented by different data types
pub trait SelectableItem: Clone {
    type SelectionValue: Clone;

    fn item_height(&self) -> u16;

    // We have four states:
    // 1. The component is focused and the item is not selected
    // 2. The component is focused and the item is selected
    // 3. The component is not focused and the item is not selected
    // 4. The component is not focused and the item is selected
    fn render(
        &self,
        is_selected: bool,
        component_focused: bool,
        width: Option<u16>,
    ) -> AnyElement<'static>;
}

pub async fn load_or_create_project(
    work_dir: &Path,
    create_if_missing: bool,
) -> Result<Option<Project>, CliError> {
    match Project::load(work_dir.to_path_buf()) {
        Ok(project) => return Ok(Some(project)),
        Err(e) => {
            info!(
                "Project not found at {}: {}. Checking if we can create one.",
                work_dir.display(),
                e
            );
        }
    }

    safety_checks(work_dir).change_context(CliError::Unknown)?;

    let mut decision = create_if_missing;
    if !decision {
        let _ = element! {
            ConfirmInputInline(
                title: format!("No project found at {}. Create it?", work_dir.display()),
                value_out: &mut decision
            )
        }
        .render_loop()
        .await;
    }

    if decision {
        init_default_project(work_dir, Some(false)).change_context(CliError::Unknown)?;
        let project = Project::load(work_dir.to_path_buf()).change_context(
            CliError::GenericError("Failed to load newly initialized project".to_string()),
        )?;
        Ok(Some(project))
    } else {
        println!("Aborting...");
        Ok(None)
    }
}
