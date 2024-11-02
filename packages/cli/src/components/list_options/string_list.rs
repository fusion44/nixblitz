use error_stack::{Result, ResultExt};
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
    string_list_popup::{PopupItem, StringListPopup},
};

#[derive(Debug, Default, Clone)]
pub struct StringListOption {
    pub value: String,
    pub selected: bool,
    pub display_name: String,
}

impl StringListOption {
    pub fn new(value: String, selected: bool, display_name: String) -> Self {
        Self {
            value,
            selected,
            display_name,
        }
    }
}

// ╭ Options (2/2) ───────────────────────────────╮
// │ BOOL TWO                                     │
// │ ✓ (true)                                     │
// ││STR LIST ONE                                 │ <- title
// ││o one                                        │ <- subtitle
// ╰──────────────────────────────────────────────╯
#[derive(Debug, Default)]
pub struct StringListOptionComponent {
    /// The title of the list item
    title: String,
    /// The subtitle of the list item
    subtitle: String,
    /// Whether this list item is selected
    selected: bool,
    dirty: bool,
    original: String,
    /// The possible options this list item can have
    options: Vec<StringListOption>,
    editing: bool,
    action_tx: Option<UnboundedSender<Action>>,
    string_list_popup: Option<Box<StringListPopup>>,
}

impl StringListOptionComponent {
    pub fn new(
        title: String,
        initial_value: String,
        selected: bool,
        options: Vec<StringListOption>,
    ) -> Self {
        Self {
            title,
            subtitle: initial_value.clone(),
            selected,
            original: initial_value.clone(),
            options,
            string_list_popup: None,
            ..Default::default()
        }
    }

    fn reset_popup(&mut self) {
        self.string_list_popup = None;
    }

    fn build_popup(&mut self) -> Result<(), CliError> {
        self.string_list_popup = Some(Box::new(StringListPopup::new(
            &self.title,
            self.options
                .iter()
                .map(|i| PopupItem {
                    value: i.value.clone(),
                    selected: i.selected,
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
        self.dirty
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
                let res = p.as_ref().selected();
                let res = res.unwrap_or_default();
                self.subtitle = self.options.get(res).unwrap().value.clone();
                self.dirty = self.original != self.subtitle;
                for opt in self.options.iter_mut() {
                    opt.selected = self.subtitle == opt.value;
                }
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
            &self.title,
            &self.subtitle,
            self.dirty,
            frame,
            area,
        )
        .change_context(CliError::UnableToDrawComponent)
        .attach_printable_lazy(|| format!("Drawing list item titled {}", self.title))?;

        if let Some(ref mut p) = self.string_list_popup {
            let _ = p.draw(frame, area, ctx);
        }

        Ok(())
    }
}
