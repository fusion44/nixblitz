use std::collections::HashMap;

use color_eyre::Result;
use ratatui::prelude::*;
use ratatui_macros::constraints;
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    action::Action,
    components::{app_list::AppList, Component},
    config::Config,
};

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
enum ComponentIndex {
    #[default]
    List,
    Options,
    Help,
}

#[derive(Default)]
pub struct AppsPage {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    app_list: AppList,
    focus: ComponentIndex,
}

impl AppsPage {
    pub fn new() -> Self {
        Self {
            command_tx: None,
            config: Config::default(),
            app_list: AppList::new(),
            focus: ComponentIndex::List,
        }
    }

    fn nav(&mut self, action: Action) {
        match action {
            Action::NavUp | Action::NavDown => {
                if self.focus == ComponentIndex::List {
                    let _ = self.app_list.update(action);
                }
            }
            Action::NavLeft => todo!(),
            Action::NavRight => todo!(),
            Action::Enter => self.on_enter(action),
            _ => (),
        }
    }

    fn on_enter(&self, action: Action) {
        // When the user hits enter and the App List is selected
        // then we'll focus on the options part of the page
        if self.focus == ComponentIndex::List {}
    }

    fn on_app_selected(&self, action: Action) {
        // TODO: implement the options component
        todo!("Implement app options");
    }
}

impl Component for AppsPage {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.app_list.register_action_handler(tx.clone())?;
        self.command_tx = Some(tx);
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        self.config = config;
        Ok(())
    }

    fn handle_mouse_event(
        &mut self,
        mouse: crossterm::event::MouseEvent,
    ) -> Result<Option<Action>> {
        self.app_list.handle_mouse_event(mouse)?;
        Ok(None)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::NavUp
            | Action::NavDown
            | Action::NavLeft
            | Action::NavRight
            | Action::Enter => self.nav(action),
            Action::AppTabAppSelected(_) => self.on_app_selected(action),
            _ => (),
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints![==20, >=0])
            .split(area);

        let _ = self.app_list.draw(frame, layout[0]);

        Ok(())
    }
}
