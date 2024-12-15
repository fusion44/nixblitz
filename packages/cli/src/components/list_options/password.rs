use error_stack::{Report, Result, ResultExt};
use nixblitzlib::{
    app_option_data::{
        option_data::{GetOptionId, OptionDataChangeNotification},
        password_data::{PasswordOptionChangeData, PasswordOptionData},
    },
    strings::OPTION_TITLES,
};
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
    password_confirm_popup::PasswordConfirmPopup,
};

#[derive(Debug, Default)]
pub struct PasswordOptionComponent<'a> {
    data: PasswordOptionData,
    title: &'a str,
    selected: bool,
    editing: bool,
    action_tx: Option<UnboundedSender<Action>>,
    popup: Option<Box<PasswordConfirmPopup<'a>>>,
}

impl<'a> PasswordOptionComponent<'a> {
    pub fn new(data: &PasswordOptionData, selected: bool) -> Result<Self, CliError> {
        let title = OPTION_TITLES
            .get(data.id())
            .ok_or(CliError::OptionTitleRetrievalError(data.id().to_string()))?;

        Ok(Self {
            data: data.clone(),
            title,
            selected,
            editing: false,
            ..Default::default()
        })
    }

    fn reset_popup(&mut self) {
        self.popup = None;
    }

    fn build_popup(&mut self) -> Result<(), CliError> {
        let mut pop = PasswordConfirmPopup::new(self.title, self.data.clone())?;
        let tx = self
            .action_tx
            .clone()
            .ok_or(CliError::UnableToFindUnboundedSender)?;
        pop.register_action_handler(tx.clone())?;
        self.popup = Some(Box::new(pop));

        Ok(())
    }

    pub fn set_data(&mut self, data: &PasswordOptionData) {
        self.data = data.clone();
    }
}

impl<'a> OptionListItem for PasswordOptionComponent<'a> {
    fn selected(&self) -> bool {
        self.selected
    }

    fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
    }

    fn is_dirty(&self) -> bool {
        self.data.dirty()
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

impl<'a> Component for PasswordOptionComponent<'a> {
    fn update(&mut self, ctx: &UpdateContext) -> Result<Option<Action>, CliError> {
        if ctx.action == Action::Esc && self.editing {
            if let Some(ref mut p) = self.popup {
                p.update(ctx)?;
            }
        } else if ctx.action == Action::PopModal(true) && self.editing {
            self.editing = false;
            if let Some(tx) = &self.action_tx {
                if let Some(p) = &self.popup {
                    let (main, confirm) = p.values();
                    tx.send(Action::AppTabOptionChangeProposal(
                        OptionDataChangeNotification::PasswordEdit(PasswordOptionChangeData::new(
                            self.data.id().clone(),
                            main,
                            Some(confirm),
                        )),
                    ))
                    .change_context(CliError::Unknown)?
                };
            }
            self.reset_popup();
        } else if ctx.action == Action::PopModal(false) && self.editing {
            self.editing = false;
            self.reset_popup();
        } else if ctx.action == Action::TogglePasswordVisibility && self.editing {
            if let Some(ref mut p) = self.popup {
                p.update(ctx)?;
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
            &self.data.subtitle(),
            self.data.dirty(),
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
