use error_stack::{Result, ResultExt};
use nixblitzlib::app_option_data::string_list_data::StringListOptionData;
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
            subtitle: data.value.clone(),
            data: data.clone(),
            selected,
            string_list_popup: None,
            ..Default::default()
        }
    }

    fn reset_popup(&mut self) {
        self.string_list_popup = None;
    }

    fn build_popup(&mut self) -> Result<(), CliError> {
        let opts = &self.data.clone().options;
        self.string_list_popup = Some(Box::new(StringListPopup::new(
            "test",
            opts.iter()
                .map(|i| SelectableListItem {
                    value: i.value.clone(),
                    selected: self.data.value == i.value,
                    display_title: i.display_name.clone(),
                })
                .collect(),
        )?));

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

    fn is_dirty(&self) -> bool {
        todo!();
    }

    fn on_edit(&mut self) -> Result<(), CliError> {
        self.editing = !self.editing;

        if self.editing {
            self.build_popup()?;
            if let Some(tx) = &self.action_tx {
                let _ = tx.send(Action::PushModal(false));
            }
        } else {
            if let Some(p) = &self.string_list_popup {
                //self.subtitle = self.data.options.get(res).unwrap().value.clone();
            }
            self.reset_popup();
            if let Some(tx) = &self.action_tx {
                let _ = tx.send(Action::PopModal(true));
            }
        }

        Ok(())
    }
}

impl Component for StringListOptionComponent {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<(), CliError> {
        self.action_tx = Some(tx);
        Ok(())
    }

    fn update(&mut self, ctx: &UpdateContext) -> Result<Option<Action>, CliError> {
        if ctx.action == Action::Esc && self.editing {
            self.editing = false;
            self.reset_popup();
            if let Some(tx) = &self.action_tx {
                let _ = tx.send(Action::PopModal(true));
            }
        } else if ctx.action == Action::NavUp
            || ctx.action == Action::NavDown
            || ctx.action == Action::PageUp
            || ctx.action == Action::PageDown && self.editing
        {
            if let Some(p) = &mut self.string_list_popup {
                return p.update(ctx);
            }
        }

        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect, ctx: &RenderContext) -> Result<(), CliError> {
        draw_item(
            self.selected,
            &self.data.value,
            &self.data.title,
            self.data.dirty,
            frame,
            area,
        )
        .change_context(CliError::UnableToDrawComponent)
        .attach_printable_lazy(|| format!("Drawing list item titled {}", self.data.title))?;

        if let Some(ref mut p) = self.string_list_popup {
            let _ = p.draw(frame, area, ctx);
        }

        Ok(())
    }
}
