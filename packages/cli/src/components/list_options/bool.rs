use error_stack::{Report, Result, ResultExt};
use nixblitzlib::app_option_data::{
    bool_data::{BoolOptionChangeData, BoolOptionData},
    option_data::OptionDataChangeNotification,
};
use ratatui::{layout::Rect, Frame};
use tokio::sync::mpsc::UnboundedSender;

use crate::{action::Action, app_contexts::RenderContext, components::Component, errors::CliError};

use super::base_option::{draw_item, OptionListItem};

#[derive(Debug, Default)]
pub struct BoolOptionComponent {
    data: BoolOptionData,
    subtitle: String,
    selected: bool,
    action_tx: Option<UnboundedSender<Action>>,
}

impl BoolOptionComponent {
    pub fn new(data: &BoolOptionData, selected: bool) -> Self {
        Self {
            data: data.clone(),
            subtitle: Self::format_subtitle(data.value),
            selected,
            action_tx: None,
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

    fn on_edit(&mut self) -> std::result::Result<(), Report<CliError>> {
        self.subtitle = Self::format_subtitle(self.data.value);
        if let Some(tx) = &self.action_tx {
            tx.send(Action::AppTabOptionChanged(
                OptionDataChangeNotification::Bool(BoolOptionChangeData::new(
                    self.data.id.clone(),
                    !self.data.value,
                )),
            ))
            .change_context(CliError::Unknown)?
        }

        Ok(())
    }

    fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
    }

    fn is_dirty(&self) -> bool {
        todo!()
    }
}

impl Component for BoolOptionComponent {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<(), CliError> {
        self.action_tx = Some(tx);
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect, _: &RenderContext) -> Result<(), CliError> {
        draw_item(
            self.selected,
            &self.data.title,
            &self.subtitle,
            self.data.dirty,
            frame,
            area,
        )
        .change_context(CliError::UnableToDrawComponent)
        .attach_printable_lazy(|| format!("Drawing list item titled {}", self.data.title))
    }
}
