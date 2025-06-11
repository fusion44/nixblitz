use error_stack::{Report, Result, ResultExt};
use nixblitz_core::{
    app_option_data::{
        option_data::{GetOptionId, OptionDataChangeNotification},
        port_data::{PortOptionChangeData, PortOptionData},
    },
    strings::OPTION_TITLES,
};
use ratatui::{Frame, layout::Rect};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    action::Action,
    app_contexts::{RenderContext, UpdateContext},
    components::Component,
    errors::CliError,
};

use super::{
    base_option::{OptionListItem, draw_item},
    number_popup::NumberInputPopup,
};

#[derive(Debug)]
pub struct PortOptionComponent<'a> {
    data: PortOptionData,
    title: &'a str,
    subtitle: String,
    selected: bool,
    editing: bool,
    action_tx: Option<UnboundedSender<Action>>,
    popup: Option<Box<NumberInputPopup<'a>>>,
}

impl PortOptionComponent<'_> {
    pub fn new(data: &PortOptionData, selected: bool) -> Result<Self, CliError> {
        let subtitle = data.value().to_string();
        let title = OPTION_TITLES
            .get(data.id())
            .ok_or(CliError::OptionTitleRetrievalError(data.id().to_string()))?;

        Ok(Self {
            data: data.clone(),
            title,
            subtitle,
            selected,
            editing: false,
            action_tx: None,
            popup: None,
        })
    }

    fn reset_popup(&mut self) {
        self.popup = None;
    }

    fn build_popup(&mut self) -> Result<(), CliError> {
        let mut pop = NumberInputPopup::new(self.title, self.data.value().clone())?;
        if let Some(h) = &self.action_tx {
            pop.register_action_handler(h.clone())?;
        }
        self.popup = Some(Box::new(pop));

        Ok(())
    }

    fn update_subtitle(&mut self) {
        self.subtitle = self.data.value().to_string();
    }

    pub fn set_data(&mut self, data: &PortOptionData) {
        self.data = data.clone();
    }
}

impl OptionListItem for PortOptionComponent<'_> {
    fn selected(&self) -> bool {
        self.selected
    }

    fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
    }

    fn is_applied(&self) -> bool {
        self.data.is_applied()
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

impl Component for PortOptionComponent<'_> {
    fn update(&mut self, ctx: &UpdateContext) -> Result<Option<Action>, CliError> {
        if ctx.action == Action::Esc && self.editing {
            if let Some(ref mut p) = self.popup {
                p.update(ctx)?;
            }
        } else if ctx.action == Action::PopModal(true) && self.editing {
            self.editing = false;
            if let Some(ref mut p) = self.popup {
                let val = p.get_result();
                self.data.set_value(val.clone());

                self.update_subtitle();

                if let Some(tx) = &self.action_tx {
                    tx.send(Action::AppTabOptionChangeProposal(
                        OptionDataChangeNotification::Port(PortOptionChangeData::new(
                            self.data.id().clone(),
                            val,
                        )),
                    ))
                    .change_context(CliError::Unknown)?
                }
            }

            self.update_subtitle();
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
            self.data.is_applied(),
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
