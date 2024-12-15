use crossterm::event::KeyCode;
use error_stack::Result;
use nixblitzlib::{app_option_data::password_data::PasswordOptionData, strings::Strings};
use ratatui::{
    layout::{Layout, Rect},
    widgets::Clear,
    Frame,
};
use ratatui_macros::constraint;
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    action::Action,
    app_contexts::{RenderContext, UpdateContext},
    components::{password_input::PasswordInput, theme::popup, Component},
    errors::CliError,
    utils::GetStringOrCliError,
};

use super::{popup::center, popup_confirm_btn_bar::PopupConfirmButtonBar};

#[derive(Debug, Default, Eq, PartialEq)]
enum PopupFocus {
    #[default]
    PW1,
    PW2,
    Accept,
    Cancel,
}

/// Represents a text input widget passwords
#[derive(Debug, Default)]
pub struct PasswordConfirmPopup<'a> {
    data: PasswordOptionData,
    title: String,
    action_tx: Option<UnboundedSender<Action>>,
    ta_main: PasswordInput<'a>,
    ta_confirm: PasswordInput<'a>,
    focus: PopupFocus,
    show_pw: bool,
    error_text_1: String,
    error_text_2: String,
}

impl PasswordConfirmPopup<'_> {
    pub fn new(title: &str, data: PasswordOptionData) -> Result<Self, CliError> {
        let main = Strings::PasswordInputPlaceholderMain.get_or_err()?;
        let ta_main = PasswordInput::new(Some(main), true, false, true)?;
        let conf = Strings::PasswordInputPlaceholderConfirm.get_or_err()?;
        let ta_confirm = PasswordInput::new(Some(conf), false, false, true)?;

        Ok(Self {
            title: format!(" {} ", title),
            data,
            focus: PopupFocus::PW1,
            ta_main,
            ta_confirm,
            ..Default::default()
        })
    }

    pub fn values(&self) -> (String, String) {
        (
            self.ta_main.lines().first().unwrap().to_string(),
            self.ta_confirm.lines().first().unwrap().to_string(),
        )
    }

    fn _set_focus(&mut self, focus: PopupFocus) -> Result<(), CliError> {
        self.focus = focus;
        match self.focus {
            PopupFocus::PW1 => {
                self.ta_main.set_focused(true);
                self.ta_confirm.set_focused(false);
            }
            PopupFocus::PW2 => {
                self.ta_main.set_focused(false);
                self.ta_confirm.set_focused(true);
            }
            PopupFocus::Accept | PopupFocus::Cancel => {
                self.ta_main.set_focused(false);
                self.ta_confirm.set_focused(false);
            }
        }

        Ok(())
    }

    fn _handle_tab(&mut self) -> Result<(), CliError> {
        match self.focus {
            PopupFocus::PW1 => self._set_focus(PopupFocus::PW2)?,
            PopupFocus::PW2 => self._set_focus(PopupFocus::Accept)?,
            PopupFocus::Accept => self._set_focus(PopupFocus::Cancel)?,
            _ => self._set_focus(PopupFocus::PW1)?,
        }

        Ok(())
    }

    fn _on_popup_confirm(&self, accepted: bool) {
        if let Some(action_tx) = &self.action_tx {
            let _ = action_tx.send(Action::PopModal(accepted));
        }
    }

    fn _verify(&mut self) -> Result<(), CliError> {
        self.error_text_1.clear();
        self.error_text_2.clear();

        let lines1 = self.ta_main.lines();
        if lines1.len() != 1 {
            self.error_text_1 = "Password must not contain newlines".into();
        }
        let line1 = lines1.first().unwrap();
        if !line1.is_empty() && line1.len() < self.data.min_length() {
            self.error_text_1 = format!(
                "Password must be at least {} characters long",
                self.data.min_length()
            );
        }

        let line2 = self.ta_confirm.lines().first().ok_or(CliError::Unknown)?;
        if !line2.is_empty() && line1 != line2 {
            self.error_text_2 = "Passwords do not match".into();
        }

        Ok(())
    }
}

impl Component for PasswordConfirmPopup<'_> {
    fn update(&mut self, ctx: &UpdateContext) -> Result<Option<Action>, CliError> {
        if ctx.action == Action::Esc {
            if let Some(action_tx) = &self.action_tx {
                let _ = action_tx.send(Action::PopModal(false));
            }
        } else if ctx.action == Action::TogglePasswordVisibility {
            self.show_pw = !self.show_pw;
            self.ta_main.set_show_password(self.show_pw);
            self.ta_confirm.set_show_password(self.show_pw);
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
        if key.code == KeyCode::Tab {
            self._handle_tab()?;
            return Ok(None);
        } else if key.code == KeyCode::Enter && self.focus == PopupFocus::PW1 {
            self.focus = PopupFocus::PW2;
            return Ok(None);
        } else if key.code == KeyCode::Enter && self.focus == PopupFocus::PW2 {
            self.focus = PopupFocus::Accept;
            return Ok(None);
        } else if key.code == KeyCode::Enter && self.focus == PopupFocus::Accept {
            self._on_popup_confirm(true);
            return Ok(None);
        } else if key.code == KeyCode::Enter && self.focus == PopupFocus::Cancel {
            self._on_popup_confirm(false);
            return Ok(None);
        }

        if self.focus == PopupFocus::PW1 {
            self.ta_main.input(key);
        } else if self.focus == PopupFocus::PW2 {
            self.ta_confirm.input(key);
        }

        self._verify()?;

        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, _: Rect, ctx: &RenderContext) -> Result<(), CliError> {
        let rect = frame.area();
        let poparea = center(frame.area(), constraint!(<=rect.width-10), constraint!(==7));

        let title = self.title.clone();
        let block = match self.focus {
            PopupFocus::PW1 | PopupFocus::PW2 => popup::block_focused(title, ctx),
            _ => popup::block(title, ctx),
        };
        let inner_layout = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                constraint!(==1),
                constraint!(==1),
                constraint!(==1),
                constraint!(==1),
                constraint!(==1),
            ])
            .split(block.inner(poparea));

        frame.render_widget(Clear, poparea);
        frame.render_widget(block, poparea);
        self.ta_main.draw(frame, inner_layout[1], ctx)?;
        if !self.error_text_1.is_empty() {
            frame.render_widget(
                popup::error_text::default(&self.error_text_1, ctx),
                inner_layout[2],
            );
        }

        self.ta_confirm.draw(frame, inner_layout[3], ctx)?;
        if !self.error_text_2.is_empty() {
            frame.render_widget(
                popup::error_text::default(&self.error_text_2, ctx),
                inner_layout[4],
            );
        }

        let btn_state = match self.focus {
            PopupFocus::PW1 | PopupFocus::PW2 => None,
            PopupFocus::Accept => Some(0),
            PopupFocus::Cancel => Some(1),
        };

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

        Ok(())
    }
}
