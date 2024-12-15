use std::{cell::RefCell, rc::Rc};

use crate::{
    action::Action,
    app_contexts::{RenderContext, UpdateContext},
    components::{app_list::AppList, app_options::AppOptions, Component},
    config::Config,
    constants::FocusableComponent,
    errors::CliError,
};

use error_stack::Result;
use nixblitzlib::{apps::SupportedApps, project::Project};
use ratatui::prelude::*;
use ratatui_macros::constraints;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Default)]
pub struct AppsPage<'a> {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    app_list: AppList,
    app_options: AppOptions<'a>,
    last_focus: FocusableComponent,
    current_focus: FocusableComponent,
}

impl<'a> AppsPage<'a> {
    pub fn new(project: Rc<RefCell<Project>>) -> Result<Self, CliError> {
        let mut instance = Self {
            command_tx: None,
            config: Config::default(),
            app_list: AppList::new(),
            app_options: AppOptions::new(project)?,
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

impl<'a> Component for AppsPage<'a> {
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

    fn update(&mut self, ctx: &UpdateContext) -> Result<Option<Action>, CliError> {
        match ctx.action {
            Action::NavUp
            | Action::NavDown
            | Action::PageUp
            | Action::PageDown
            | Action::TogglePasswordVisibility => {
                if ctx.modal_open {
                    return self.app_options.update(ctx);
                }

                if self.current_focus == FocusableComponent::AppTabList {
                    return self.app_list.update(ctx);
                } else if self.current_focus == FocusableComponent::AppTabOptions {
                    return self.app_options.update(ctx);
                }
            }
            Action::NavLeft | Action::NavRight => todo!(),
            Action::Enter => {
                // When the user hits enter and the App List is selected
                // then we'll focus on the options part of the page
                if self.current_focus == FocusableComponent::AppTabList {
                    self.on_focus_req(FocusableComponent::AppTabOptions);
                } else if self.current_focus == FocusableComponent::AppTabOptions {
                    return self.app_options.update(ctx);
                }
            }
            Action::Esc => {
                if ctx.modal_open {
                    return self.app_options.update(ctx);
                }

                if self.current_focus == FocusableComponent::AppTabOptions {
                    self.on_focus_req(FocusableComponent::AppTabList);
                }
            }
            Action::AppTabOptionChangeAccepted => {
                return self.app_options.update(ctx);
            }
            Action::AppTabAppSelected(app) => self.on_app_selected(app),
            Action::FocusRequest(r) => self.on_focus_req(r),
            Action::PopModal(_) => {
                self.app_options.update(ctx)?;
            }
            _ => (),
        }

        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect, ctx: &RenderContext) -> Result<(), CliError> {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints![==20, >=25])
            .split(area);

        self.app_list.draw(frame, layout[0], ctx)?;
        self.app_options.draw(frame, layout[1], ctx)?;

        Ok(())
    }
}
