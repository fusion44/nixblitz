use error_stack::{Result, ResultExt};
use nixblitzlib::{
    app_option_data::{
        option_data::{GetOptionId, OptionDataChangeNotification},
        string_list_data::{StringListOptionChangeData, StringListOptionData},
    },
    strings::OPTION_TITLES,
};
use ratatui::{layout::Rect, Frame};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    action::Action,
    app_contexts::{RenderContext, UpdateContext},
    components::{theme::list::SelectableListItem, Component},
    errors::CliError,
};

use super::{
    base_option::{draw_item, OptionListItem},
    string_list_popup::StringListPopup,
};

// ╭ Options (2/2) ───────────────────────────────╮
// │ BOOL TWO                                     │
// │ ✓ (true)                                     │
// ││STR LIST ONE                                 │ <- title
// ││o one                                        │ <- subtitle
// ╰──────────────────────────────────────────────╯
#[derive(Debug, Default)]
pub struct StringListOptionComponent {
    /// The subtitle of the list item
    subtitle: String,
    data: StringListOptionData,
    selected: bool,
    editing: bool,
    action_tx: Option<UnboundedSender<Action>>,
    string_list_popup: Option<Box<StringListPopup>>,
}

impl StringListOptionComponent {
    pub fn new(data: &StringListOptionData, selected: bool) -> Self {
        Self {
            subtitle: data.value().to_string(),
            data: data.clone(),
            selected,
            string_list_popup: None,
            ..Default::default()
        }
    }

    pub fn set_data(&mut self, data: &StringListOptionData) {
        self.data = data.clone();
    }

    fn handle_edit_start(&mut self) -> Result<(), CliError> {
        self.editing = true;
        let opts = self.data.options();
        let tx = &self
            .action_tx
            .clone()
            .ok_or(CliError::UnableToFindUnboundedSender)?;

        self.string_list_popup = Some(Box::new(StringListPopup::new(
            "test",
            opts.iter()
                .map(|i| SelectableListItem {
                    value: i.value.clone(),
                    selected: self.data.value() == i.value,
                    display_title: i.display_name.clone(),
                })
                .collect(),
            tx.clone(),
        )?));

        tx.send(Action::PushModal(false))
            .change_context(CliError::UnableToFindUnboundedSender)?;

        Ok(())
    }

    fn handle_edit_end(&mut self, edit_accepted: bool) -> Result<(), CliError> {
        if !edit_accepted {
            self.string_list_popup = None;
            self.editing = false;

            return Ok(());
        }

        let p = &self
            .string_list_popup
            .as_mut()
            .ok_or(CliError::Unknown)
            .attach_printable_lazy(|| "Popup is None when edit state is true")?;

        let selected_item = p
            .selected()
            .ok_or(CliError::Unknown)
            .attach_printable_lazy(|| "Unable to get the selected option id from popup")?;

        let options = self.data.options();
        let selected_item = options
            .get(selected_item)
            .ok_or(CliError::Unknown)
            .attach_printable_lazy(|| "Unable to get the selected option from options list")?;

        let tx = self
            .action_tx
            .clone()
            .ok_or(CliError::UnableToFindUnboundedSender)?;

        tx.send(Action::AppTabOptionChangeProposal(
            OptionDataChangeNotification::StringList(StringListOptionChangeData::new(
                self.data.id().clone(),
                selected_item.value.clone(),
            )),
        ))
        .change_context(CliError::UnableToSendViaUnboundedSender)?;

        self.string_list_popup = None;
        self.editing = false;

        Ok(())
    }
}

impl OptionListItem for StringListOptionComponent {
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
        // Do nothing if we are already in edit mode
        // Update will handle the reset
        if self.editing {
            return Ok(());
        }
        self.handle_edit_start()?;

        Ok(())
    }
}

impl Component for StringListOptionComponent {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<(), CliError> {
        self.action_tx = Some(tx);
        Ok(())
    }

    fn update(&mut self, ctx: &UpdateContext) -> Result<Option<Action>, CliError> {
        if let Action::PopModal(value) = ctx.action {
            if self.editing {
                self.handle_edit_end(value)?;
            }
        } else if self.editing {
            let p = &mut self
                .string_list_popup
                .as_mut()
                .ok_or(CliError::Unknown)
                .attach_printable("Unable to find popup, even though we are editing")?;

            return p.update(ctx);
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
            self.data.value(),
            self.data.is_applied(),
            frame,
            area,
        )
        .change_context(CliError::UnableToDrawComponent)
        .attach_printable_lazy(|| format!("Drawing list item titled {}", title))?;

        if let Some(ref mut p) = self.string_list_popup {
            let _ = p.draw(frame, area, ctx);
        }

        Ok(())
    }
}
