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

#[derive(Default)]
pub struct AppsPage {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    app_list: AppList,
    app_options: AppOptions,
    last_focus: FocusableComponent,
    current_focus: FocusableComponent,
}

impl AppsPage {
    pub fn new() -> Result<Self, CliError> {
        let mut instance = Self {
            command_tx: None,
            config: Config::default(),
            app_list: AppList::new(),
            app_options: AppOptions::new()?,
            current_focus: FocusableComponent::AppTabList,
            ..Default::default()
        };
        instance.on_focus_req(FocusableComponent::AppTabList);
        Ok(instance)
    }

    fn on_app_selected(&mut self, app: SupportedApps) {
        self.app_options.set_app(app);
    }

    fn on_focus_req(&mut self, c: FocusableComponent) {
        self.last_focus = self.current_focus;
        self.current_focus = c;
        match c {
            FocusableComponent::AppTabList => {
                self.app_list.set_focus(true);
                self.app_options.set_focus(false);
            }
            FocusableComponent::AppTabOptions => {
                self.app_list.set_focus(false);
                self.app_options.set_focus(true);
            }
            FocusableComponent::AppTabHelp => (),
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

    fn handle_key_event(
        &mut self,
        key: crossterm::event::KeyEvent,
    ) -> Result<Option<Action>, CliError> {
        self.app_options.handle_key_event(key)
    }

    fn update(&mut self, action: Action, modal_open: bool) -> Result<Option<Action>, CliError> {
        match action {
            Action::NavUp | Action::NavDown | Action::PageUp | Action::PageDown => {
                if modal_open {
                    return self.app_options.update(action.clone(), modal_open);
                }

                if self.current_focus == FocusableComponent::AppTabList {
                    return self.app_list.update(action, modal_open);
                } else if self.current_focus == FocusableComponent::AppTabOptions {
                    return self.app_options.update(action, modal_open);
                }
            }
            Action::NavLeft | Action::NavRight => todo!(),
            Action::Enter => {
                // When the user hits enter and the App List is selected
                // then we'll focus on the options part of the page
                if self.current_focus == FocusableComponent::AppTabList {
                    self.on_focus_req(FocusableComponent::AppTabOptions);
                } else if self.current_focus == FocusableComponent::AppTabOptions {
                    let _res = self.app_options.on_enter();
                }
            }
            Action::Esc => {
                if modal_open {
                    return self.app_options.update(action, modal_open);
                }

                if self.current_focus == FocusableComponent::AppTabOptions {
                    self.on_focus_req(FocusableComponent::AppTabList);
                }
            }
            Action::AppTabAppSelected(app) => self.on_app_selected(app),
            Action::FocusRequest(r) => self.on_focus_req(r),
            Action::PopModal(_) => {
                self.app_options.update(action, modal_open)?;
            }
            _ => (),
        }

        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect, modal_open: bool) -> Result<(), CliError> {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints![==20, >=25])
            .split(area);

        self.app_list.draw(frame, layout[0], modal_open)?;
        self.app_options.draw(frame, layout[1], modal_open)?;

        Ok(())
    }
}
