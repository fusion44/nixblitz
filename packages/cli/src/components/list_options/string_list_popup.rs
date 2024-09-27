use error_stack::{Report, Result};
use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Padding},
    Frame,
};
use ratatui_macros::constraint;

use crate::{
    action::{self, Action},
    colors,
    components::Component,
    errors::CliError,
};

/// Represents an item within a Popup menu.
#[derive(Debug)]
pub struct PopupItem {
    /// The underlying value associated with the item.
    pub value: String,

    /// Indicates whether the item is currently selected.
    pub selected: bool,

    /// The title displayed for the item in the Popup menu.
    pub display_title: String,
}

/// Represents a Popup menu widget for string lists.
#[derive(Debug, Default)]
pub struct StringListPopup {
    /// The title displayed at the top of the Popup menu.
    title: String,

    /// The list of items contained within the Popup menu.
    options: Vec<PopupItem>,

    /// Maintains the current selection state within the Popup menu.
    state: ListState,

    /// Number of items in the options.
    max_len: u16,
}

impl StringListPopup {
    /// Constructs a new Popup menu for string lists.
    ///
    /// # Arguments
    /// * `title` - The title of the Popup menu.
    /// * `options` - The list of items for the Popup menu.
    /// * `fixed` - Whether the Popup menu should have a fixed width.
    ///
    /// # Returns
    /// A `Result` containing the constructed `Popup` or a `CliError`
    /// if the maximum title length exceeds 128 characters.
    pub fn new(title: &str, options: Vec<PopupItem>) -> Result<Self, CliError> {
        let mut selected_id = 0;
        let max_len = options
            .iter()
            .enumerate()
            .map(|(i, e)| {
                if e.selected && i > selected_id {
                    selected_id = i;
                }
                e.display_title.len()
            })
            .max()
            .unwrap_or(0);

        if max_len > 128 {
            return Err(Report::new(CliError::MaxDisplayNameLengthReached)
                .attach_printable(format!("Max: 128; Actual: {}", max_len)));
        }

        let mut state = ListState::default();
        state.select(Some(selected_id));
        Ok(Self {
            title: format!(" {} ", title),
            options,
            state,
            max_len: max_len as u16,
        })
    }

    pub fn selected(&self) -> Option<usize> {
        self.state.selected()
    }
}

impl Component for StringListPopup {
    fn update(
        &mut self,
        action: action::Action,
        _: bool,
    ) -> Result<Option<action::Action>, CliError> {
        let pos = self.state.selected();
        if pos.is_none() {
            self.state.select(Some(0));
        }

        match action {
            Action::NavUp => self.state.select_previous(),
            Action::NavDown => self.state.select_next(),
            Action::PageUp => self.state.scroll_up_by(10),
            Action::PageDown => self.state.scroll_down_by(10),
            _ => (),
        }

        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, _: Rect, _: bool) -> error_stack::Result<(), CliError> {
        assert!(u16::try_from(self.options.len()).is_ok());

        let height: u16 = self.options.len() as u16 + 2;
        let width: u16 = self.max_len + 12;

        let poparea = center(frame.area(), constraint!(==width), constraint!(==height));

        let block = Block::default()
            .title(self.title.as_str())
            .title_alignment(ratatui::layout::Alignment::Center)
            .padding(Padding::horizontal(1))
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(Style::new().fg(colors::CYAN_700));

        let list_items: Vec<ListItem> = self.options.iter().map(ListItem::from).collect();
        let list = List::new(list_items)
            .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
            .highlight_symbol("|")
            .block(block);

        frame.render_widget(Clear, poparea);
        frame.render_stateful_widget(list, poparea, &mut self.state);

        Ok(())
    }
}

fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);

    area
}

impl From<&PopupItem> for ListItem<'_> {
    fn from(value: &PopupItem) -> Self {
        let line = match value.selected {
            false => Line::styled(format!(" ☐ {}", value.display_title), colors::WHITE),
            true => Line::styled(format!(" ✓ {}", value.display_title), colors::CYAN_500),
        };
        ListItem::new(line)
    }
}
