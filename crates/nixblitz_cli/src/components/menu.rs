use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use error_stack::{Result, ResultExt};
use ratatui::{Frame, layout::Rect};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    action::Action,
    app_contexts::{RenderContext, UpdateContext},
    config::Config,
    errors::CliError,
};

use super::{Component, theme::menu};

const MARGIN: u16 = 2;

#[derive(Copy, Clone, Debug, Default)]
pub enum MenuItem {
    #[default]
    Apps,
    Settings,
    Actions,
    Help,
}

impl MenuItem {
    pub fn get_splits(&self) -> &[usize] {
        match self {
            MenuItem::Apps => &[1, 4],
            MenuItem::Settings => &[1, 4],
            MenuItem::Actions => &[1, 5],
            MenuItem::Help => &[1, 4],
        }
    }
}

impl From<MenuItem> for usize {
    fn from(value: MenuItem) -> Self {
        match value {
            MenuItem::Apps => 0,
            MenuItem::Settings => 1,
            MenuItem::Actions => 2,
            MenuItem::Help => 3,
        }
    }
}

impl From<&str> for MenuItem {
    fn from(value: &str) -> Self {
        match value {
            "[1] Apps" => MenuItem::Apps,
            "[2] Settings" => MenuItem::Settings,
            "[3] Actions" => MenuItem::Actions,
            "[4] Help" => MenuItem::Help,
            _ => MenuItem::Apps,
        }
    }
}

// We know that the height of a hitbox of a menu entry is only
// one line, so we store only the start and the end of the entry
//
// RaspiBlitz | Apps | Settings | Actions | Help
//1234567890123
//   Offset   |M    M|M        M|M       M|M    M
#[derive(Debug, Default)]
struct Hitbox {
    start: u16,
    end: u16,
}

#[derive(Debug, Default)]
struct MenuEntry {
    item: MenuItem,
    title: String,
    hitbox: Hitbox,
}

#[derive(Debug, Default)]
pub struct Menu {
    command_tx: Option<UnboundedSender<Action>>,
    active_item: MenuItem,
    config: Config,
    event: Option<MouseEvent>,
    entries: Vec<MenuEntry>,
}

impl Menu {
    pub fn new(offset: u16) -> Self {
        let mut instance = Self::default();
        let entries = ["[1] Apps", "[2] Settings", "[3] Actions", "[4] Help"];

        let mut curr = offset;
        for entry in entries {
            let new_entry = MenuEntry {
                item: MenuItem::from(entry),
                title: entry.to_string(),
                hitbox: Hitbox {
                    start: curr,
                    end: curr + entry.len() as u16 + MARGIN,
                },
            };
            curr = new_entry.hitbox.end + 1;
            instance.entries.push(new_entry);
        }

        instance
    }

    pub fn set_active_item(&mut self, item: MenuItem) {
        self.active_item = item;
    }
}

impl Component for Menu {
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
        self.event = Some(mouse);
        Ok(None)
    }

    fn update(&mut self, ctx: &UpdateContext) -> Result<Option<Action>, CliError> {
        match ctx.action {
            Action::NavAppsTab => self.set_active_item(MenuItem::Apps),
            Action::NavSettingsTab => self.set_active_item(MenuItem::Settings),
            Action::NavActionsTab => self.set_active_item(MenuItem::Actions),
            Action::NavHelpTab => self.set_active_item(MenuItem::Help),
            _ => {}
        }

        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect, ctx: &RenderContext) -> Result<(), CliError> {
        let items: Vec<_> = self
            .entries
            .iter()
            .map(|t| menu::item(t.title.as_str(), t.item.get_splits(), ctx))
            .collect();

        let tabs = menu::tab_bar(items, self.active_item.into(), ctx);
        if let Some(mouse) = self.event {
            if mouse.kind == MouseEventKind::Up(MouseButton::Left)
                && mouse.row == area.y
                && mouse.column >= area.x
            {
                let mx = mouse.column;
                for entry in &self.entries {
                    if entry.hitbox.start <= mx && entry.hitbox.end >= mx {
                        if let Some(tx) = &self.command_tx {
                            let _ = tx
                                .send(match entry.item {
                                    MenuItem::Apps => Action::NavAppsTab,
                                    MenuItem::Settings => Action::NavSettingsTab,
                                    MenuItem::Actions => Action::NavActionsTab,
                                    MenuItem::Help => Action::NavHelpTab,
                                })
                                .attach_printable_lazy(|| "Unable to send mouse action")
                                .change_context(CliError::Unknown);
                        }
                    }
                }
            }

            self.event = None;
        }
        frame.render_widget(tabs, area);

        Ok(())
    }
}
