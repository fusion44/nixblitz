use crossterm::event::KeyCode;
use error_stack::Result;
use log::warn;
use nixblitzlib::{number_value::NumberValue, strings::DECIMAL_SIGN};
use ratatui::{layout::Rect, widgets::Clear, Frame};
use ratatui_macros::constraint;
use tokio::sync::mpsc::UnboundedSender;
use tui_textarea::TextArea;

use crate::{
    action::Action,
    app_contexts::{RenderContext, UpdateContext},
    components::{theme::popup, Component},
    errors::CliError,
};

use super::popup::center;

/// Represents a text input widget for single and multi line strings.
#[derive(Debug)]
pub struct NumberInputPopup<'a> {
    title: String,
    value: NumberValue,
    text_area: TextArea<'a>,
    action_tx: Option<UnboundedSender<Action>>,
}

impl NumberInputPopup<'_> {
    pub fn new(title: &str, value: NumberValue) -> Result<Self, CliError> {
        let lines = [value.to_string()].to_vec();
        Ok(Self {
            title: format!(" {} ", title),
            value,
            text_area: TextArea::new(lines),
            action_tx: None,
        })
    }

    pub fn get_result(&mut self) -> NumberValue {
        let value = self.text_area.lines().first();
        if let Some(value) = value {
            if value.is_empty() {
                return self.value.as_none();
            }

            let value = value.parse::<f64>();
            if let Ok(value) = value {
                self.value.set_value(Some(value));
                return self.value.clone();
            } else {
                warn!(
                    "NumberInputPopup: Invalid value: {}",
                    self.text_area.lines().join("\n")
                );
            }
        }

        self.value.as_none()
    }

    fn _on_popup_confirm(&self, accepted: bool) {
        if let Some(action_tx) = &self.action_tx {
            let _ = action_tx.send(Action::PopModal(accepted));
        }
    }
}

impl Component for NumberInputPopup<'_> {
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
        // TODO: improve the input handling
        if key.code == KeyCode::Enter {
            self._on_popup_confirm(true);
            return Ok(None);
        } else if let KeyCode::Char(c) = key.code {
            if c.is_ascii_digit() || c == DECIMAL_SIGN {
                self.text_area.input(key);
            }
        } else if key.code == KeyCode::Backspace
            || key.code == KeyCode::Delete
            || key.code == KeyCode::Left
            || key.code == KeyCode::Right
        {
            self.text_area.input(key);
        }

        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, _: Rect, ctx: &RenderContext) -> Result<(), CliError> {
        let rect = frame.area();
        let poparea = center(frame.area(), constraint!(<=rect.width-10), constraint!(==3));

        let title = self.title.clone();
        let block = popup::block_focused(title, ctx);
        self.text_area.set_block(block);

        frame.render_widget(Clear, poparea);
        frame.render_widget(&self.text_area, poparea);

        Ok(())
    }
}
