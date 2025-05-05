use crate::{
    action::Action,
    app_contexts::{RenderContext, UpdateContext},
    components::{theme, Component},
    config::Config,
    errors::CliError,
};
use error_stack::Result;
use ratatui::prelude::*;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Default)]
pub struct SettingsPage {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
}

impl SettingsPage {
    pub fn new() -> Self {
        Self {
            command_tx: None,
            config: Config::default(),
        }
    }

    fn nav(&mut self, action: &Action) {
        match action {
            Action::NavUp | Action::NavDown => {}
            Action::Enter => self.on_enter(),
            Action::Esc => self.on_esc(),
            _ => (),
        }
    }

    fn on_enter(&mut self) {}

    fn on_esc(&mut self) {}
}

impl Component for SettingsPage {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<(), CliError> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> Result<(), CliError> {
        self.config = config;
        Ok(())
    }

    fn handle_mouse_event(
        &mut self,
        mouse: crossterm::event::MouseEvent,
    ) -> Result<Option<Action>, CliError> {
        let _ = mouse;
        Ok(None)
    }

    fn update(&mut self, ctx: &UpdateContext) -> Result<Option<Action>, CliError> {
        match ctx.action {
            Action::NavUp
            | Action::NavDown
            | Action::NavLeft
            | Action::NavRight
            | Action::Enter
            | Action::Esc => self.nav(&ctx.action),
            _ => (),
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect, ctx: &RenderContext) -> Result<(), CliError> {
        let c = theme::block::default(" Settings ", ctx);
        frame.render_widget(c, area);

        Ok(())
    }
}
