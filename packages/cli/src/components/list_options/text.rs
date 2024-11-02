use error_stack::{Report, Result, ResultExt};
use ratatui::{layout::Rect, Frame};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    action::Action,
    app_contexts::{RenderContext, UpdateContext},
    components::Component,
    errors::CliError,
};

use super::{
    base_option::{draw_item, OptionListItem},
    text_popup::TextInputPopup,
};

#[derive(Debug, Default)]
pub struct TextOptionComponent<'a> {
    title: &'a str,
    subtitle: String,
    value: String,
    selected: bool,
    dirty: bool,
    original: String,

    max_lines: u16,
    editing: bool,
    action_tx: Option<UnboundedSender<Action>>,
    popup: Option<Box<TextInputPopup<'a>>>,
}

impl<'a> TextOptionComponent<'a> {
    pub fn new(
        title: &'a str,
        initial_value: String,
        selected: bool,
        max_lines: u16,
    ) -> Result<Self, CliError> {
        Ok(Self {
            title,
            subtitle: initial_value.clone(),
            value: initial_value.clone(),
            selected,
            dirty: false,
            original: initial_value,
            editing: false,
            max_lines,
            ..Default::default()
        })
    }

    fn reset_popup(&mut self) {
        self.popup = None;
    }

    fn build_popup(&mut self) -> Result<(), CliError> {
        let mut pop = TextInputPopup::new(
            self.title,
            self.value.lines().map(String::from).collect(),
            self.max_lines,
        )?;
        if let Some(h) = &self.action_tx {
            pop.register_action_handler(h.clone())?;
        }
        self.popup = Some(Box::new(pop));

        Ok(())
    }

    fn check_dirty(&mut self) {
        self.dirty = self.original != self.value;
    }

    fn update_subtitle(&mut self) {
        if let Some(first_line) = self.value.lines().next() {
            self.subtitle = first_line.to_string();
        }
    }
}

impl<'a> OptionListItem for TextOptionComponent<'a> {
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
        if !self.editing {
            self.editing = !self.editing;
            self.build_popup()?;
            if let Some(tx) = &self.action_tx {
                let _ = tx.send(Action::PushModal(true));
            }
        }

        Ok(())
    }
}

impl<'a> Component for TextOptionComponent<'a> {
    fn update(&mut self, ctx: &UpdateContext) -> Result<Option<Action>, CliError> {
        if ctx.action == Action::Esc && self.editing {
            if let Some(ref mut p) = self.popup {
                p.update(ctx)?;
            }
        } else if ctx.action == Action::PopModal(true) && self.editing {
            self.editing = false;
            if let Some(ref mut p) = self.popup {
                match self.max_lines {
                    1 => {
                        self.value = p.get_result()[0].clone();
                        self.subtitle = self.value.clone();
                    }
                    n if n > 1 => {
                        self.value = p.get_result().join("\n");
                    }
                    _ => {}
                }
            }

            self.update_subtitle();
            self.check_dirty();
            self.reset_popup();
        } else if ctx.action == Action::PopModal(false) && self.editing {
            self.editing = false;
            self.reset_popup();
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
        if !self.editing {
            return Ok(None);
        }

        if let Some(ref mut p) = self.popup {
            return p.handle_key_event(key);
        }

        Ok(None)
    }
    fn draw(&mut self, frame: &mut Frame, area: Rect, ctx: &RenderContext) -> Result<(), CliError> {
        draw_item(
            self.selected,
            self.title,
            &self.subtitle,
            self.dirty,
            frame,
            area,
        )
        .change_context(CliError::UnableToDrawComponent)
        .attach_printable_lazy(|| format!("Drawing list item titled {}", self.title))?;

        if let Some(ref mut p) = self.popup {
            p.draw(frame, area, ctx)?;
        }

        Ok(())
    }
}
