use std::{cell::RefCell, collections::HashMap, path::PathBuf, rc::Rc};

use crossterm::event::KeyEvent;
use error_stack::{Report, Result, ResultExt};
use log::{error, info, trace};
use nixblitzlib::project::Project;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    prelude::Rect,
    Frame,
};
use ratatui_macros::constraints;
use serde::{Deserialize, Serialize};
use strum::Display;
use tokio::sync::mpsc;

use crate::{
    action::Action,
    app_contexts::{RenderContext, UpdateContext},
    components::{
        menu::Menu,
        theme::{self, ThemeData},
        title::Title,
        Component,
    },
    config::Config,
    errors::CliError,
    pages::{
        actions_page::ActionsPage, apps_page::AppsPage, help_page::HelpPage,
        settings_page::SettingsPage,
    },
    tui::{Event, Tui},
};

static APP_TITLE: &str = " RaspiBlitz |";

pub struct App {
    config: Config,
    tick_rate: f64,
    frame_rate: f64,
    components_map: HashMap<ComponentIndex, Box<dyn Component>>,
    should_quit: bool,
    should_suspend: bool,
    mode: Mode,
    last_tick_key_events: Vec<KeyEvent>,
    action_tx: mpsc::UnboundedSender<Action>,
    action_rx: mpsc::UnboundedReceiver<Action>,
    home_page: ComponentIndex,
    dirty: bool,
    theme: Rc<RefCell<ThemeData>>,
    project: Rc<RefCell<Project>>,

    /// Tracks if a modal is open
    modal_open: bool,

    /// Tracks whether this modal has a text area
    /// this will direct all input to this modal
    exclusive_input_component_shown: bool,
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Mode {
    #[default]
    Home,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Display)]
enum ComponentIndex {
    Menu,
    Title,
    AppsPage,
    SettingsPage,
    ActionsPage,
    HelpPage,
}

impl App {
    pub fn new(tick_rate: f64, frame_rate: f64, work_dir: PathBuf) -> Result<Self, CliError> {
        trace!("Creating new App instance");
        let project =
            Project::load(work_dir).change_context(CliError::UnableToInitProjectStruct)?;
        let project = Rc::new(RefCell::new(project));
        info!("Project loaded successfully");

        let (action_tx, action_rx) = mpsc::unbounded_channel();

        info!("Initializing UI components");
        let mut map: HashMap<ComponentIndex, Box<dyn Component>> = HashMap::new();
        map.insert(
            ComponentIndex::Title,
            Box::new(Title::new(APP_TITLE.to_string())),
        );
        map.insert(
            ComponentIndex::Menu,
            Box::new(Menu::new(APP_TITLE.len() as u16)),
        );
        map.insert(
            ComponentIndex::AppsPage,
            Box::new(AppsPage::new(project.clone())?),
        );
        map.insert(ComponentIndex::SettingsPage, Box::new(SettingsPage::new()));
        map.insert(ComponentIndex::ActionsPage, Box::new(ActionsPage::new()));
        map.insert(ComponentIndex::HelpPage, Box::new(HelpPage::new()));
        trace!("All UI components initialized");

        trace!("Creating config");
        let config = Config::new()
            .attach_printable_lazy(|| "Unable to create new config")
            .change_context(CliError::Unknown)?;

        info!("App initialization complete");
        Ok(Self {
            tick_rate,
            frame_rate,
            components_map: map,
            should_quit: false,
            should_suspend: false,
            config,
            mode: Mode::Home,
            last_tick_key_events: Vec::new(),
            action_tx,
            action_rx,
            home_page: ComponentIndex::AppsPage,
            project,
            modal_open: false,
            exclusive_input_component_shown: false,
            dirty: true,
            theme: Rc::new(RefCell::new(ThemeData::default())),
        })
    }

    pub async fn run(&mut self) -> Result<(), CliError> {
        info!("Running app");
        self.theme.borrow_mut().set_theme("pale-green", "dark")?;

        trace!("Creating and configuring TUI");
        let mut tui = Tui::new()?
            .mouse(true)
            .tick_rate(self.tick_rate)
            .frame_rate(self.frame_rate);

        trace!("Entering TUI alternate screen");
        tui.enter()?;

        info!("Registering component handlers");
        for component in self.components_map.iter_mut() {
            component
                .1
                .register_action_handler(self.action_tx.clone())?;
        }
        for component in self.components_map.iter_mut() {
            component.1.register_config_handler(self.config.clone())?;
        }

        info!("Initializing component layouts");
        for component in self.components_map.iter_mut() {
            let size = tui
                .size()
                .attach_printable_lazy(|| "Unable to init the first component")
                .change_context(CliError::Unknown)?;
            let r = Rect::new(0, 0, size.width, size.height);
            component.1.init(r)?;
        }

        trace!("Entering main event loop");
        let action_tx = self.action_tx.clone();
        loop {
            self.handle_events(&mut tui).await?;
            self.handle_actions(&mut tui)?;
            if self.should_suspend {
                trace!("Suspending TUI");
                tui.suspend()?;
                action_tx
                    .send(Action::Resume)
                    .attach_printable_lazy(|| "Unable to send the Resume action")
                    .change_context(CliError::Unknown)?;
                action_tx
                    .send(Action::ClearScreen)
                    .attach_printable_lazy(|| "Unable to send the clear screen action")
                    .change_context(CliError::Unknown)?;

                trace!("Re-entering TUI");
                tui.enter()?;
            } else if self.should_quit {
                info!("Quit signal received, stopping TUI");
                tui.stop()?;
                break;
            }
        }

        info!("Exiting TUI and cleanup");
        tui.exit()?;
        Ok(())
    }

    async fn handle_events(&mut self, tui: &mut Tui) -> Result<(), CliError> {
        let Some(event) = tui.next_event().await else {
            return Ok(());
        };
        let action_tx = self.action_tx.clone();
        match event {
            Event::Quit => action_tx
                .send(Action::Quit)
                .change_context(CliError::Unknown)?,
            Event::Tick => action_tx
                .send(Action::Tick)
                .change_context(CliError::Unknown)?,
            Event::Render => action_tx
                .send(Action::Render)
                .change_context(CliError::Unknown)?,
            Event::Resize(x, y) => action_tx
                .send(Action::Resize(x, y))
                .change_context(CliError::Unknown)?,
            Event::Key(key) => self.handle_key_event(key)?,
            _ => (),
        }
        for component in self.components_map.iter_mut() {
            if let Some(action) = component.1.handle_events(Some(event.clone()))? {
                action_tx.send(action).change_context(CliError::Unknown)?;
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<(), CliError> {
        self.dirty = true;
        let Some(keymap) = self.config.keybindings.get(&self.mode) else {
            return Ok(());
        };

        match keymap.get(&vec![key]) {
            Some(action) => {
                self.action_tx
                    .clone()
                    .send(action.clone())
                    .change_context(CliError::Unknown)?;
            }
            _ => {
                // If the key was not handled as a single key action,
                // then consider it for multi-key combinations.
                self.last_tick_key_events.push(key);

                // Check for multi-key combinations
                if let Some(action) = keymap.get(&self.last_tick_key_events) {
                    self.action_tx
                        .clone()
                        .send(action.clone())
                        .change_context(CliError::Unknown)?;
                }
            }
        }
        Ok(())
    }

    fn handle_actions(&mut self, tui: &mut Tui) -> Result<(), CliError> {
        while let Ok(action) = self.action_rx.try_recv() {
            if action == Action::Render && self.dirty {
                self.render(tui)?;
                self.dirty = false;
                continue;
            } else if action == Action::Render && !self.dirty {
                continue;
            }

            match action.clone() {
                Action::Tick => {
                    self.last_tick_key_events.drain(..);
                }
                Action::Quit => self.on_quit(),
                Action::Suspend => self.should_suspend = true,
                Action::Resume => self.should_suspend = false,
                Action::ClearScreen => tui.terminal.clear().change_context(CliError::Unknown)?,
                Action::Resize(w, h) => self.handle_resize(tui, w, h)?,
                Action::NavAppsTab
                | Action::NavSettingsTab
                | Action::NavActionsTab
                | Action::NavHelpTab => {
                    // Don't navigate or forward the event if a modal is opened
                    if self.modal_open {
                        continue;
                    }

                    self.handle_tab_nav(&action);
                }
                Action::PushModal(_) | Action::PopModal(_) => self.handle_modal_change(&action)?,
                Action::AppTabOptionChangeProposal(opt) => {
                    let updated = self
                        .project
                        .borrow_mut()
                        .on_option_changed(opt)
                        .change_context(CliError::Unknown)?;

                    if updated {
                        self.dirty = true;
                        self.action_tx
                            .send(Action::AppTabOptionChangeAccepted)
                            .change_context(CliError::UnableToSendViaUnboundedSender)?;
                        self.action_tx
                            .send(Action::Render)
                            .change_context(CliError::UnableToSendViaUnboundedSender)?;
                    }
                }
                Action::TogglePasswordVisibility => {
                    self.dirty = true;
                    self.action_tx
                        .send(Action::AppTabOptionChangeAccepted)
                        .change_context(CliError::UnableToSendViaUnboundedSender)?;
                }
                Action::AppTabAppSelected(app) => {
                    self.project.borrow_mut().set_selected_app(app);
                    self.dirty = true;
                }
                _ => {}
            }

            if self.exclusive_input_component_shown {
                match action {
                    Action::Esc | Action::PopModal(_) | Action::TogglePasswordVisibility => {}
                    _ => {
                        continue;
                    }
                }
            }

            let ctx = UpdateContext::new(action.clone(), self.modal_open, self.project.clone());
            for component in self.components_map.iter_mut() {
                if let Some(action) = component.1.update(&ctx)? {
                    self.action_tx
                        .send(action)
                        .change_context(CliError::Unknown)?
                };
            }
        }
        Ok(())
    }

    fn on_quit(&mut self) {
        if self.modal_open {
            return;
        }

        self.should_quit = true;
    }

    fn handle_tab_nav(&mut self, action: &Action) {
        match action {
            Action::NavAppsTab => self.home_page = ComponentIndex::AppsPage,
            Action::NavSettingsTab => self.home_page = ComponentIndex::SettingsPage,
            Action::NavActionsTab => self.home_page = ComponentIndex::ActionsPage,
            Action::NavHelpTab => self.home_page = ComponentIndex::HelpPage,
            _ => (),
        }
    }

    fn handle_resize(&mut self, tui: &mut Tui, w: u16, h: u16) -> Result<(), CliError> {
        tui.resize(Rect::new(0, 0, w, h))
            .change_context(CliError::Unknown)?;
        self.render(tui)?;
        Ok(())
    }

    fn draw_app_bar(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        ctx: &RenderContext,
    ) -> Result<(), CliError> {
        let menu_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Length(APP_TITLE.len().try_into().unwrap()),
                    Constraint::Min(0),
                ]
                .as_ref(),
            )
            .split(area);

        // Draw the title
        self.components_map
            .get_mut(&ComponentIndex::Title)
            .unwrap()
            .draw(frame, menu_layout[0], ctx)?;

        // Draw the menu{
        self.components_map
            .get_mut(&ComponentIndex::Menu)
            .unwrap()
            .draw(frame, menu_layout[1], ctx)?;

        Ok(())
    }

    fn draw_app_body(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        ctx: &RenderContext,
    ) -> Result<(), CliError> {
        if self.home_page == ComponentIndex::AppsPage {
            self.components_map
                .get_mut(&ComponentIndex::AppsPage)
                .unwrap()
                .draw(frame, area, ctx)?;
        } else if self.home_page == ComponentIndex::SettingsPage {
            self.components_map
                .get_mut(&ComponentIndex::SettingsPage)
                .unwrap()
                .draw(frame, area, ctx)?;
        } else if self.home_page == ComponentIndex::ActionsPage {
            self.components_map
                .get_mut(&ComponentIndex::ActionsPage)
                .unwrap()
                .draw(frame, area, ctx)?;
        } else if self.home_page == ComponentIndex::HelpPage {
            self.components_map
                .get_mut(&ComponentIndex::HelpPage)
                .unwrap()
                .draw(frame, area, ctx)?;
        } else {
            error!("Unknown home page {}", self.home_page);
        }

        Ok(())
    }

    fn render(&mut self, tui: &mut Tui) -> Result<(), CliError> {
        let ctx = RenderContext::new(self.modal_open, self.theme.clone(), self.project.clone());

        tui.draw(|frame| {
            let main_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(constraints![==1, *=1, ==2])
                .split(frame.area());

            let res = self.draw_app_bar(frame, main_layout[0], &ctx);
            if let Err(e) = res {
                error!("{}", e);
            }

            let res = self.draw_app_body(frame, main_layout[1], &ctx);
            if let Err(e) = res {
                error!("{}", e);
            }
            frame.render_widget(theme::block::no_border(&ctx), main_layout[2]);
        })
        .attach_printable_lazy(|| "Unable to draw the frame")
        .change_context(CliError::Unknown)?;

        Ok(())
    }

    fn handle_modal_change(&mut self, action: &Action) -> Result<(), CliError> {
        match action {
            Action::PushModal(v) => {
                if self.modal_open {
                    Err(Report::new(CliError::MultipleModalsOpened))?;
                }
                self.modal_open = true;
                self.exclusive_input_component_shown = *v;
            }
            Action::PopModal(_success) => {
                self.modal_open = false;
                self.exclusive_input_component_shown = false;
            }
            _ => Err(Report::new(CliError::Unknown)
                .attach_printable(format!("Receives action wrong {}", action)))?,
        }

        trace!(
            "Handle modal change. Modals open: {}; has_text_area: {}",
            self.modal_open,
            self.exclusive_input_component_shown
        );

        Ok(())
    }
}
