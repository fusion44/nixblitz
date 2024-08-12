use super::{container::render_container, Component};
use crate::{action::Action, config::Config, constants::FocusableComponent};
use color_eyre::Result;
use crossterm::event::{MouseButton, MouseEventKind};

use nixbitcfg::apps::SupportedApps;
use ratatui::{prelude::*, widgets::*};
use tokio::sync::mpsc::UnboundedSender;

const APP_TITLE: &str = " Apps ";

#[derive(Default)]
pub struct AppList {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    state: ListState,
    mouse_click_pos: Option<Position>,
    focus: bool,
}

impl AppList {
    pub fn new() -> Self {
        let mut instance = Self::default();
        instance.state.select(Some(0));
        instance
    }

    /// Checks if mouse click is within the specified area and returns the index of
    /// the selected row (zero based), or `None` if not applicable.
    ///
    /// # Parameters:
    /// - `area`: Rectangle representing the check area.
    ///
    /// # Returns:
    /// - The index of the clicked row, with Some(0) being the first item
    /// - Otherwise, returns `None`.
    fn check_user_mouse_select(&mut self, area: Rect) -> Option<usize> {
        if let Some(c) = self.mouse_click_pos {
            self.mouse_click_pos = None;

            if area.contains(c) {
                let res = (c.y - area.y) as usize;
                if res == 0 {
                    return None;
                }

                // subtract one to account for the
                // block decoration
                return Some(res - 1);
            }
        }

        None
    }

    fn kb_select_item(&mut self, action: Action) {
        let pos = self.state.selected();
        if pos.is_none() {
            self.state.select(Some(0));
            self.send_selected_action(0);
        }

        if action == Action::NavUp {
            self.state.select_previous();
        } else if action == Action::NavDown {
            self.state.select_next();
        }

        if pos != self.state.selected() {
            self.send_selected_action(self.state.selected().unwrap());
        }
    }

    fn mouse_select_item(&mut self, pos: usize) {
        let old = self.state.selected();
        if pos > SupportedApps::as_string_list().len() {
            return;
        }

        if Some(pos) != old {
            self.state.select(Some(pos));
            self.send_selected_action(pos);
        }
    }

    fn send_selected_action(&mut self, pos: usize) {
        if let Some(tx) = &self.command_tx {
            let res = SupportedApps::from_id(pos);
            if let Some(app) = res {
                let _ = tx.send(Action::AppTabAppSelected(app));
            }
        }
    }

    fn send_focus_req_action(&mut self) {
        if let Some(tx) = &self.command_tx {
            let _ = tx.send(Action::FocusRequest(FocusableComponent::AppTabList));
        }
    }

    fn render_app_list(&mut self, frame: &mut Frame, area: Rect) {
        let list = List::new(SupportedApps::as_string_list().to_owned())
            .block(render_container(APP_TITLE, self.focus))
            .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
            .highlight_symbol(">>")
            .repeat_highlight_symbol(true);

        frame.render_stateful_widget(list, area, &mut self.state);
    }

    pub fn set_focus(&mut self, focus: bool) {
        self.focus = focus;
    }
}

impl Component for AppList {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
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
        if mouse.kind == MouseEventKind::Up(MouseButton::Left) {
            self.mouse_click_pos = Some(Position::new(mouse.column, mouse.row));
        }

        Ok(None)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::NavUp | Action::NavDown => self.kb_select_item(action),
            _ => (),
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let res = self.check_user_mouse_select(area);
        if let Some(pos) = res {
            self.send_focus_req_action();
            self.mouse_select_item(pos);
        }

        self.render_app_list(frame, area);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_check_user_mouse_select() {
        let mut app = AppList::new();
        app.mouse_click_pos = None;
        assert_eq!(app.check_user_mouse_select(Rect::default()), None);

        // pub enum SupportedApps {
        //     #[default]
        //     BitcoinCore,     index 0
        //     CoreLightning,   index 1
        //     ...
        // }

        // A click to 5, 2 should yield Core Lightning, or index 1
        // ╭ Apps ────────────╮   (5, 0)
        // │>>Bitcoin Core    │   (5, 1)
        // │  Core Lightning  │   (5, 2)
        // │  LND             │   (5, 3)
        app.mouse_click_pos = Position::new(5, 2).into();
        let res = app.check_user_mouse_select(Rect::new(0, 0, 10, 10));
        assert_eq!(res, Some(1));

        app.mouse_select_item(res.unwrap());
        let selected_id = app.state.selected().unwrap();
        assert_eq!(
            SupportedApps::from_id(selected_id),
            Some(SupportedApps::CoreLightning)
        );
    }
}
