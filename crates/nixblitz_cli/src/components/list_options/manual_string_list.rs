use nixblitz_core::{
    app_option_data::{
        manual_string_list_data::{ManualStringListOptionChangeData, ManualStringListOptionData},
        option_data::{GetOptionId, OptionDataChangeNotification},
    },
    strings::OPTION_TITLES,
};

use error_stack::{Result, ResultExt};
use log::error;
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
    text_popup::TextInputPopup,
};

#[derive(Debug, Default)]
pub struct ManualStringListOptionComponent<'a> {
    subtitle: String,
    selected: bool,
    data: ManualStringListOptionData,
    editing: bool,
    action_tx: Option<UnboundedSender<Action>>,
    popup: Option<Box<TextInputPopup<'a>>>,
}

impl ManualStringListOptionComponent<'_> {
    pub fn new(data: &ManualStringListOptionData, selected: bool) -> Self {
        let subtitle: String = data.value().first().cloned().unwrap_or_default();

        Self {
            subtitle,
            selected,
            data: data.clone(),
            popup: None,
            ..Default::default()
        }
    }

    pub fn set_data(&mut self, data: &ManualStringListOptionData) {
        self.data = data.clone();
    }
    fn handle_edit_start(&mut self) -> Result<(), CliError> {
        let title =
            OPTION_TITLES
                .get(self.data.id())
                .ok_or(CliError::OptionTitleRetrievalError(
                    self.data.id().to_string(),
                ))?;

        let mut pop = TextInputPopup::new(title, self.data.value().clone(), self.data.max_lines())?;

        if let Some(tx) = &self.action_tx {
            pop.register_action_handler(tx.clone())?;
            let _ = tx.send(Action::PushModal(true));
        } else {
            error!("Unable to find action tx");
        }
        self.popup = Some(Box::new(pop));
        self.editing = true;

        Ok(())
    }

    fn handle_edit_end(&mut self, edit_accepted: bool) -> Result<(), CliError> {
        if !edit_accepted {
            self.popup = None;
            self.editing = false;

            return Ok(());
        }

        let p = &mut self
            .popup
            .as_mut()
            .ok_or(CliError::Unknown)
            .attach_printable_lazy(|| "Popup is None when edit state is true")?;

        let items: Vec<String> = p
            .get_result()
            .iter()
            .filter(|&s| !s.is_empty())
            .map(|s| s.to_string())
            .collect::<Vec<_>>();

        self.subtitle = items.first().map(|s| s.to_string()).unwrap_or_default();

        let tx = self
            .action_tx
            .clone()
            .ok_or(CliError::UnableToFindUnboundedSender)?;

        tx.send(Action::AppTabOptionChangeProposal(
            OptionDataChangeNotification::ManualStringList(ManualStringListOptionChangeData::new(
                self.data.id().clone(),
                items,
            )),
        ))
        .change_context(CliError::UnableToSendViaUnboundedSender)?;

        self.popup = None;
        self.editing = false;

        Ok(())
    }
}

impl OptionListItem for ManualStringListOptionComponent<'_> {
    fn selected(&self) -> bool {
        self.selected
    }

    fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
    }

    fn is_applied(&self) -> bool {
        self.data.is_applied()
    }

    fn on_edit(&mut self) -> Result<(), CliError> {
        if self.editing {
            return Ok(());
        }
        self.handle_edit_start()?;

        Ok(())
    }
}

impl Component for ManualStringListOptionComponent<'_> {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<(), CliError> {
        self.action_tx = Some(tx);
        Ok(())
    }

    fn update(&mut self, ctx: &UpdateContext) -> Result<Option<Action>, CliError> {
        if ctx.action == Action::Esc && self.editing {
            if let Some(ref mut p) = self.popup {
                p.update(ctx)?;
            }
        } else if ctx.action == Action::PopModal(true) && self.editing {
            self.handle_edit_end(true)?;
        } else if ctx.action == Action::PopModal(false) && self.editing {
            self.handle_edit_end(false)?;
        }

        Ok(None)
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
        let title =
            OPTION_TITLES
                .get(self.data.id())
                .ok_or(CliError::OptionTitleRetrievalError(
                    self.data.id().to_string(),
                ))?;
        draw_item(
            self.selected,
            title,
            &self.subtitle,
            self.data.is_applied(),
            frame,
            area,
        )
        .change_context(CliError::UnableToDrawComponent)
        .attach_printable_lazy(|| format!("Drawing list item titled {}", title))?;

        if let Some(ref mut p) = self.popup {
            p.draw(frame, area, ctx)?;
        }

        Ok(())
    }
}
