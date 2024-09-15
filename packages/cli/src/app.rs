use std::{collections::HashMap, path::PathBuf};

use crossterm::event::KeyEvent;
use error_stack::{Result, ResultExt};
use nixblitzlib::system::System;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    prelude::Rect,
    Frame,
};
use ratatui_macros::constraints;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::{
    action::Action,
    components::{menu::Menu, title::Title, Component},
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
    system: System,
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Mode {
    #[default]
    Home,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
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
        let system = System::new(work_dir);

        let (action_tx, action_rx) = mpsc::unbounded_channel();
        let mut map: HashMap<ComponentIndex, Box<dyn Component>> = HashMap::new();
        map.insert(
            ComponentIndex::Title,
            Box::new(Title::new(APP_TITLE.to_string())),
        );
        map.insert(
            ComponentIndex::Menu,
            Box::new(Menu::new(APP_TITLE.len() as u16)),
        );
        map.insert(ComponentIndex::AppsPage, Box::new(AppsPage::new()));
        map.insert(ComponentIndex::SettingsPage, Box::new(SettingsPage::new()));
        map.insert(ComponentIndex::ActionsPage, Box::new(ActionsPage::new()));
        map.insert(ComponentIndex::HelpPage, Box::new(HelpPage::new()));

        Ok(Self {
            tick_rate,
            frame_rate,
            components_map: map,
            should_quit: false,
            should_suspend: false,
            config: Config::new()
                .attach_printable_lazy(|| "Unable to create new config")
                .change_context(CliError::Unknown)?,
            mode: Mode::Home,
            last_tick_key_events: Vec::new(),
            action_tx,
            action_rx,
            home_page: ComponentIndex::AppsPage,
            system,
        })
    }

    pub async fn run(&mut self) -> Result<(), CliError> {
        let mut tui = Tui::new()?
            .mouse(true)
            .tick_rate(self.tick_rate)
            .frame_rate(self.frame_rate);
        tui.enter()?;

        for component in self.components_map.iter_mut() {
            component
                .1
                .register_action_handler(self.action_tx.clone())?;
        }
        for component in self.components_map.iter_mut() {
            component.1.register_config_handler(self.config.clone())?;
        }
        for component in self.components_map.iter_mut() {
            component.1.init(
                tui.size()
                    .attach_printable_lazy(|| "Unable to init the first component")
                    .change_context(CliError::Unknown)?,
            )?;
        }

        self.system.init().change_context(CliError::Unknown)?;

        let action_tx = self.action_tx.clone();
        loop {
            self.handle_events(&mut tui).await?;
            self.handle_actions(&mut tui)?;
            if self.should_suspend {
                tui.suspend()?;
                action_tx
                    .send(Action::Resume)
                    .attach_printable_lazy(|| "Unable to send the Resume action")
                    .change_context(CliError::Unknown)?;
                action_tx
                    .send(Action::ClearScreen)
                    .attach_printable_lazy(|| "Unable to send the clear screen action")
                    .change_context(CliError::Unknown)?;

                // tui.mouse(true);
                tui.enter()?;
            } else if self.should_quit {
                tui.stop()?;
                break;
            }
        }
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
        let action_tx = self.action_tx.clone();
        let Some(keymap) = self.config.keybindings.get(&self.mode) else {
            return Ok(());
        };
        match keymap.get(&vec![key]) {
            Some(action) => {
                action_tx
                    .send(action.clone())
                    .change_context(CliError::Unknown)?;
            }
            _ => {
                // If the key was not handled as a single key action,
                // then consider it for multi-key combinations.
                self.last_tick_key_events.push(key);

                // Check for multi-key combinations
                if let Some(action) = keymap.get(&self.last_tick_key_events) {
                    action_tx
                        .send(action.clone())
                        .change_context(CliError::Unknown)?;
                }
            }
        }
        Ok(())
    }

    fn handle_actions(&mut self, tui: &mut Tui) -> Result<(), CliError> {
        while let Ok(action) = self.action_rx.try_recv() {
            match action {
                Action::Tick => {
                    self.last_tick_key_events.drain(..);
                }
                Action::Quit => self.should_quit = true,
                Action::Suspend => self.should_suspend = true,
                Action::Resume => self.should_suspend = false,
                Action::ClearScreen => tui.terminal.clear().change_context(CliError::Unknown)?,
                Action::Resize(w, h) => self.handle_resize(tui, w, h)?,
                Action::NavAppsTab
                | Action::NavSettingsTab
                | Action::NavActionsTab
                | Action::NavHelpTab => self.handle_tab_nav(&action),
                Action::Render => self.render(tui)?,
                _ => {}
            }
            for component in self.components_map.iter_mut() {
                if let Some(action) = component.1.update(action.clone())? {
                    self.action_tx
                        .send(action)
                        .change_context(CliError::Unknown)?
                };
            }
        }
        Ok(())
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

    fn draw_app_bar(&mut self, frame: &mut Frame, area: Rect) {
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
        let res = self
            .components_map
            .get_mut(&ComponentIndex::Title)
            .unwrap()
            .draw(frame, menu_layout[0]);

        if let Err(err) = res {
            let _ = self
                .action_tx
                .send(Action::Error(format!("Failed to draw: {:?}", err)));
        }

        // Draw the menu
        let res = self
            .components_map
            .get_mut(&ComponentIndex::Menu)
            .unwrap()
            .draw(frame, menu_layout[1]);

        if let Err(err) = res {
            let _ = self
                .action_tx
                .send(Action::Error(format!("Failed to draw: {:?}", err)));
        }
    }

    fn draw_app_body(&mut self, frame: &mut Frame, area: Rect) {
        let res;
        if self.home_page == ComponentIndex::AppsPage {
            res = self
                .components_map
                .get_mut(&ComponentIndex::AppsPage)
                .unwrap()
                .draw(frame, area);
        } else if self.home_page == ComponentIndex::SettingsPage {
            res = self
                .components_map
                .get_mut(&ComponentIndex::SettingsPage)
                .unwrap()
                .draw(frame, area);
        } else if self.home_page == ComponentIndex::ActionsPage {
            res = self
                .components_map
                .get_mut(&ComponentIndex::ActionsPage)
                .unwrap()
                .draw(frame, area);
        } else if self.home_page == ComponentIndex::HelpPage {
            res = self
                .components_map
                .get_mut(&ComponentIndex::HelpPage)
                .unwrap()
                .draw(frame, area);
        } else {
            todo!();
        }

        if let Err(err) = res {
            let _ = self
                .action_tx
                .send(Action::Error(format!("Failed to draw: {:?}", err)));
        }
    }

    fn render(&mut self, tui: &mut Tui) -> Result<(), CliError> {
        tui.draw(|frame| {
            let main_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(constraints![==3, *=1, ==5])
                .split(frame.size());

            self.draw_app_bar(frame, main_layout[0]);
            self.draw_app_body(frame, main_layout[1]);
        })
        .attach_printable_lazy(|| "Unable to draw the frame")
        .change_context(CliError::Unknown)?;
        Ok(())
    }
}
