use super::{container::render_container, Component};
use crate::{action::Action, config::Config, constants::FocusableComponent};
use color_eyre::Result;
use crossterm::event::{MouseButton, MouseEventKind};

use nixbitcfg::apps::SupportedApps;
use ratatui::{prelude::*, widgets::*};
use tokio::sync::mpsc::UnboundedSender;
use tui_scrollview::ScrollViewState;

const TITLE: &str = " Options ";

#[derive(Default)]
pub struct AppOptions {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    mouse_click_pos: Option<Position>,
    focus: bool,
    scroll_view_state: ScrollViewState,
    app: SupportedApps,
}

impl AppOptions {
    pub fn new() -> Self {
        Self::default()
    }

    fn check_user_mouse_select(&mut self, area: Rect) -> Option<usize> {
        if let Some(c) = self.mouse_click_pos {
            self.mouse_click_pos = None;

            if area.contains(c) {
                return Some((c.y - area.y) as usize);
            }
        }

        None
    }

    fn kb_select_item(&mut self, action: Action) {}

    fn mouse_select_item(&mut self, pos: usize) {}

    fn send_focus_req_action(&mut self) {
        if let Some(tx) = &self.command_tx {
            let _ = tx.send(Action::FocusRequest(FocusableComponent::AppTabOptions));
        }
    }

    fn render_app_list(&mut self, frame: &mut Frame, area: Rect) {
        let p = Paragraph::new(format!("Hello {}.", self.app.to_string()))
            .block(render_container(TITLE, self.focus));

        frame.render_widget(p, area);
    }

    pub fn set_focus(&mut self, focus: bool) {
        self.focus = focus;
    }

    pub fn set_app(&mut self, app: SupportedApps) {
        self.app = app;
    }
}

impl Component for AppOptions {
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
