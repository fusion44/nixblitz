use crossterm::event::KeyCode;
use error_stack::{Result, ResultExt};
use ratatui::{
    Frame,
    layout::{Margin, Rect},
    widgets::{Clear, Scrollbar, ScrollbarOrientation, ScrollbarState},
};
use ratatui_macros::constraint;
use tokio::sync::mpsc::UnboundedSender;
use tui_textarea::TextArea;

use crate::{
    action::Action,
    app_contexts::{RenderContext, UpdateContext},
    components::{Component, theme::popup},
    errors::CliError,
};

use super::{popup::center, popup_confirm_btn_bar::PopupConfirmButtonBar};

#[derive(Debug, Default, Eq, PartialEq)]
enum PopupFocus {
    #[default]
    Edit,
    Accept,
    Cancel,
}

/// Represents a text input widget for single and multi line strings.
#[derive(Debug, Default)]
pub struct TextInputPopup<'a> {
    title: String,
    max_lines: u16,
    num_lines: u16,
    text_area: TextArea<'a>,
    scrollbar_state: ScrollbarState,
    action_tx: Option<UnboundedSender<Action>>,
    cursor_pos: usize,
    focus: PopupFocus,
}

impl TextInputPopup<'_> {
    pub fn new(title: &str, lines: Vec<String>, max_lines: u16) -> Result<Self, CliError> {
        Ok(Self {
            title: format!(" {} ", title),
            text_area: TextArea::new(lines.clone()),
            max_lines,
            num_lines: 0,
            ..Default::default()
        })
    }

    pub fn get_result(&mut self) -> Vec<String> {
        self.text_area.lines().to_vec()
    }

    fn update_lines(&mut self) -> Result<bool, CliError> {
        let old = self.num_lines;
        self.num_lines = u16::try_from(self.text_area.lines().len())
            .attach_printable("text area contains more lines than allowed")
            .change_context(CliError::ArgumentError)?;

        Ok(old != self.num_lines)
    }

    fn _handle_tab(&mut self) {
        match self.focus {
            PopupFocus::Edit => self.focus = PopupFocus::Accept,
            PopupFocus::Accept => self.focus = PopupFocus::Cancel,
            _ => self.focus = PopupFocus::Edit,
        }
    }

    fn _on_popup_confirm(&self, accepted: bool) {
        if let Some(action_tx) = &self.action_tx {
            let _ = action_tx.send(Action::PopModal(accepted));
        }
    }
}

impl Component for TextInputPopup<'_> {
    fn update(&mut self, ctx: &UpdateContext) -> Result<Option<Action>, CliError> {
        if ctx.action == Action::Esc {
            if let Some(action_tx) = &self.action_tx {
                let _ = action_tx.send(Action::PopModal(false));
            }
        }

        Ok(None)
    }
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<(), CliError> {
        self.action_tx = Some(tx);
        Ok(())
    }

    fn handle_key_event(
        &mut self,
        key: crossterm::event::KeyEvent,
    ) -> Result<Option<Action>, CliError> {
        let num_lines = self.text_area.lines().len();
        if key.code == KeyCode::Enter && self.max_lines == 1 {
            self._on_popup_confirm(true);
            return Ok(None);
        } else if key.code == KeyCode::Tab && self.max_lines > 1 {
            self._handle_tab();
            return Ok(None);
        } else if key.code == KeyCode::Enter
            && self.focus == PopupFocus::Edit
            && num_lines == usize::from(self.max_lines)
        {
            return Ok(None);
        } else if key.code == KeyCode::Enter && self.focus == PopupFocus::Accept {
            self._on_popup_confirm(true);
            return Ok(None);
        } else if key.code == KeyCode::Enter && self.focus == PopupFocus::Cancel {
            self._on_popup_confirm(false);
            return Ok(None);
        }

        self.text_area.input(key);
        let (row, _) = self.text_area.cursor();
        self.cursor_pos = row;
        let _ = self.update_lines();

        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, _: Rect, ctx: &RenderContext) -> Result<(), CliError> {
        let rect = frame.area();
        // calculate the max height we can use as a base
        let mut height: u16 = frame.area().height - 16;
        let mut show_scrollbar: bool = false;
        if self.max_lines == 1 {
            // single line popup
            height = 3;
        } else if self.max_lines > 1 && self.max_lines <= height {
            // multiple lines, but fits inside the available space
            // set the popup height to the number of available lines
            height = self.max_lines + 2;
        } else if self.max_lines > height && self.num_lines > height - 4 {
            // max lines is greater than the available height, but the
            // actual number of lines is less than the available space
            show_scrollbar = true;
        }

        let poparea = center(
            frame.area(),
            constraint!(<=rect.width-10),
            constraint!(==height),
        );

        let title = self.title.clone();
        let block = match self.focus {
            PopupFocus::Edit => popup::block_focused(title, ctx),
            _ => popup::block(title, ctx),
        };
        self.text_area.set_block(block);

        frame.render_widget(Clear, poparea);
        frame.render_widget(&self.text_area, poparea);
        if show_scrollbar {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"));
            self.scrollbar_state = self
                .scrollbar_state
                .content_length(self.num_lines.into())
                .viewport_content_length(height.into())
                .position(self.cursor_pos);
            frame.render_stateful_widget(
                scrollbar,
                poparea.inner(Margin {
                    // using an inner vertical margin of 1 unit makes the scrollbar inside the block
                    vertical: 1,
                    horizontal: 0,
                }),
                &mut self.scrollbar_state,
            );
        }
        let btn_state = match self.focus {
            PopupFocus::Edit => None,
            PopupFocus::Accept => Some(0),
            PopupFocus::Cancel => Some(1),
        };

        if self.max_lines > 1 {
            let mut bar =
                PopupConfirmButtonBar::new(btn_state, ["ACCEPT".into(), "CANCEL".into()].to_vec())?;
            bar.draw(
                frame,
                Rect {
                    x: poparea.left(),
                    y: poparea.bottom(),
                    width: poparea.width,
                    height: 1,
                },
                ctx,
            )?;
        }

        Ok(())
    }
}
