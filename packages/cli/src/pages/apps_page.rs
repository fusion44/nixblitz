use crate::{
    action::Action,
    components::{app_list::AppList, app_options::AppOptions, Component},
    config::Config,
    constants::FocusableComponent,
    errors::CliError,
};

use error_stack::Result;
use nixblitzlib::apps::SupportedApps;
use ratatui::prelude::*;
use ratatui_macros::constraints;
use tokio::sync::mpsc::UnboundedSender;

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
    app_options: AppOptions,
    current_focus: ComponentIndex,
}

impl AppsPage {
    pub fn new() -> Self {
        let mut instance = Self {
            command_tx: None,
            config: Config::default(),
            app_list: AppList::new(),
            app_options: AppOptions::new(),
            current_focus: ComponentIndex::List,
        };
        instance.change_focus(ComponentIndex::List);
        instance
    }

    fn nav(&mut self, action: Action) {
        match action {
            Action::NavUp | Action::NavDown => {
                if self.current_focus == ComponentIndex::List {
                    let _ = self.app_list.update(action);
                } else if self.current_focus == ComponentIndex::Options {
                    let _ = self.app_options.update(action);
                }
            }
            Action::NavLeft => todo!(),
            Action::NavRight => todo!(),
            Action::Enter => self.on_enter(),
            Action::Esc => self.on_esc(),
            _ => (),
        }
    }

    fn on_enter(&mut self) {
        // When the user hits enter and the App List is selected
        // then we'll focus on the options part of the page

        if self.current_focus == ComponentIndex::List {
            self.change_focus(ComponentIndex::Options);
        }
    }

    fn on_esc(&mut self) {
        if self.current_focus == ComponentIndex::Options {
            self.change_focus(ComponentIndex::List);
        }
    }

    fn on_app_selected(&mut self, app: SupportedApps) {
        self.app_options.set_app(app);
    }

    fn change_focus(&mut self, index: ComponentIndex) {
        self.current_focus = index;
        match index {
            ComponentIndex::List => {
                self.app_list.set_focus(true);
                self.app_options.set_focus(false);
            }
            ComponentIndex::Options => {
                self.app_list.set_focus(false);
                self.app_options.set_focus(true);
            }
            ComponentIndex::Help => (),
        }
    }

    fn on_focus_req(&mut self, c: FocusableComponent) {
        match c {
            FocusableComponent::AppTabList => self.change_focus(ComponentIndex::List),
            FocusableComponent::AppTabOptions => self.change_focus(ComponentIndex::Options),
            FocusableComponent::AppTabHelp => self.change_focus(ComponentIndex::Help),
            _ => (),
        }
    }
}

impl Component for AppsPage {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<(), CliError> {
        self.app_list.register_action_handler(tx.clone())?;
        self.app_options.register_action_handler(tx.clone())?;
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
        self.app_list.handle_mouse_event(mouse)?;
        self.app_options.handle_mouse_event(mouse)?;
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
            Action::AppTabAppSelected(app) => self.on_app_selected(app),
            Action::FocusRequest(r) => self.on_focus_req(r),
            _ => (),
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<(), CliError> {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints![==20, >=25])
            .split(area);

        let _ = self.app_list.draw(frame, layout[0]);
        let _ = self.app_options.draw(frame, layout[1]);

        Ok(())
    }
}
