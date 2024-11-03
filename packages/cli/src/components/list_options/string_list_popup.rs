use error_stack::{Report, Result};
use ratatui::{
    layout::Rect,
    widgets::{Clear, ListState},
    Frame,
};
use ratatui_macros::constraint;

use crate::{
    action::{self, Action},
    app_contexts::{RenderContext, UpdateContext},
    components::{
        list_options::popup::center,
        theme::{
            block,
            list::{self, SelectableListItem},
            popup::{self, block},
        },
        Component,
    },
    errors::CliError,
};

/// Represents a Popup menu widget for string lists.
#[derive(Debug, Default)]
pub struct StringListPopup {
    /// The title displayed at the top of the Popup menu.
    title: String,

    /// The list of items contained within the Popup menu.
    options: Vec<SelectableListItem>,

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
    pub fn new(title: &str, options: Vec<SelectableListItem>) -> Result<Self, CliError> {
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
    fn update(&mut self, ctx: &UpdateContext) -> Result<Option<action::Action>, CliError> {
        let pos = self.state.selected();
        if pos.is_none() {
            self.state.select(Some(0));
        }

        match ctx.action {
            Action::NavUp => self.state.select_previous(),
            Action::NavDown => self.state.select_next(),
            Action::PageUp => self.state.scroll_up_by(10),
            Action::PageDown => self.state.scroll_down_by(10),
            _ => (),
        }

        Ok(None)
    }

    fn draw(
        &mut self,
        frame: &mut Frame,
        _: Rect,
        ctx: &RenderContext,
    ) -> error_stack::Result<(), CliError> {
        assert!(u16::try_from(self.options.len()).is_ok());

        let height: u16 = self.options.len() as u16 + 2;
        let width: u16 = self.max_len + 12;

        let poparea = center(frame.area(), constraint!(==width), constraint!(==height));
        let block = popup::block_focused(self.title.clone(), ctx);
        let list = list::select::default(&self.options, ctx).block(block);

        frame.render_widget(Clear, poparea);
        frame.render_stateful_widget(list, poparea, &mut self.state);

        Ok(())
    }
}
