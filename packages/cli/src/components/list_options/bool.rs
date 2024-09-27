use error_stack::{Report, Result, ResultExt};
use ratatui::{layout::Rect, Frame};

use crate::{components::Component, errors::CliError};

use super::base_option::{draw_item, OptionListItem};

#[derive(Debug, Default)]
pub struct BoolOptionComponent {
    title: String,
    subtitle: String,
    value: bool,
    selected: bool,
    dirty: bool,
    original: bool,
}

impl BoolOptionComponent {
    pub fn new(title: &str, initial_value: bool, selected: bool) -> Self {
        Self {
            title: String::from(title),
            subtitle: Self::format_subtitle(initial_value),
            value: initial_value,
            selected,
            dirty: false,
            original: initial_value,
        }
    }

    fn format_subtitle(value: bool) -> String {
        if value {
            return "✓ (true)".to_string();
        }

        "✗ (false)".to_string()
    }
}

impl OptionListItem for BoolOptionComponent {
    fn selected(&self) -> bool {
        self.selected
    }

    fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
    }

    fn is_dirty(&self) -> bool {
        self.dirty
    }

    fn on_edit(&mut self) -> std::result::Result<(), Report<CliError>> {
        self.value = !self.value;
        self.dirty = self.value != self.original;
        self.subtitle = Self::format_subtitle(self.value);

        Ok(())
    }
}

impl Component for BoolOptionComponent {
    fn draw(&mut self, frame: &mut Frame, area: Rect, _: bool) -> Result<(), CliError> {
        draw_item(
            self.selected,
            &self.title,
            &self.subtitle,
            self.dirty,
            frame,
            area,
        )
        .change_context(CliError::UnableToDrawComponent)
        .attach_printable_lazy(|| format!("Drawing list item titled {}", self.title))
    }
}
