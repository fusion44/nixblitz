use ratatui::{layout::Rect, Frame};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    action::Action,
    components::{container::render_container, Component},
    config::Config,
    errors::CliError,
};

#[derive(Default)]
pub struct HelpPage {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
}

impl HelpPage {
    pub fn new() -> Self {
        Self {
            command_tx: None,
            config: Config::default(),
        }
    }

    fn nav(&mut self, action: Action) {
        match action {
            Action::NavUp | Action::NavDown => {}
            Action::NavLeft => todo!(),
            Action::NavRight => todo!(),
            Action::Enter => self.on_enter(),
            Action::Esc => self.on_esc(),
            _ => (),
        }
    }

    fn on_enter(&mut self) {}

    fn on_esc(&mut self) {}
}

impl Component for HelpPage {
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
        Ok(None)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>, CliError> {
        match action {
            Action::NavUp
            | Action::NavDown
            | Action::NavLeft
            | Action::NavRight
            | Action::Enter
            | Action::Esc => self.nav(action),
            _ => (),
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<(), CliError> {
        let c = render_container(" Help ", true);
        frame.render_widget(c, area);

        Ok(())
    }
}
