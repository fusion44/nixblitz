use super::{
    container::render_container,
    option::{ListOption, OptionType},
    Component,
};
use crate::{action::Action, config::Config, constants::FocusableComponent, errors::CliError};
use crossterm::event::{MouseButton, MouseEventKind};

use nixblitzlib::apps::SupportedApps;
use ratatui::prelude::*;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Default)]
pub struct AppOptions {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    mouse_click_pos: Option<Position>,
    focus: bool,
    options: Vec<ListOption>,
    constraints: Vec<Constraint>,
    app: SupportedApps,
    selected: usize,
    offset: usize,
    max_num_items: usize,
    title: String,
}

impl AppOptions {
    pub fn new() -> Self {
        let items: Vec<ListOption> = (0..35)
            .map(|i| {
                ListOption::new(
                    format!("Test {}", i).to_string(),
                    OptionType::Bool(true),
                    false,
                )
            })
            .collect();

        let cons = (0..items.len()).map(|_| Constraint::Length(2)).collect();
        let mut item = Self {
            options: items,
            constraints: cons,
            ..Self::default()
        };
        item.select(0);
        item
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

    fn kb_select_item(&mut self, action: Action) {
        match action {
            Action::NavUp => self.select_previous(),
            Action::NavDown => self.select_next(),
            _ => (),
        }
    }

    fn mouse_select_item(&mut self, pos: usize) {
        let _ = pos;
    }

    fn send_focus_req_action(&mut self) {
        if let Some(tx) = &self.command_tx {
            let _ = tx.send(Action::FocusRequest(FocusableComponent::AppTabOptions));
        }
    }

    fn render_options_list(&mut self, frame: &mut Frame, area: Rect) {
        let block = render_container(&self.title, self.focus);
        let total_height = block.inner(area).height;
        self.max_num_items = (total_height / 2) as usize;

        assert!(
            total_height % 2 == 0,
            "Area height must be a multiple of two for now.\ntotal_height: {}\nmax_num_items: {}",
            total_height,
            self.max_num_items
        );

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                self.constraints
                    .iter()
                    .skip(self.offset)
                    .take(self.max_num_items),
            )
            .split(block.inner(area));
        frame.render_widget(block, area);

        for (index, value) in self
            .options
            .iter_mut()
            .skip(self.offset)
            .enumerate()
            .take(self.max_num_items)
        {
            let _ = value.draw(frame, layout[index]);
        }
    }

    pub fn set_focus(&mut self, focus: bool) {
        self.focus = focus;
    }

    pub fn set_app(&mut self, app: SupportedApps) {
        self.app = app;
    }

    fn select_previous(&mut self) {
        if self.selected == 0 {
            self.offset = 0;
            return;
        }

        // Check if we have to scroll
        if self.offset > 0 && self.offset == self.selected {
            self.offset -= 1;
        }

        let new_selected = self.selected - 1;
        // Get the old selected item
        let res = self.options.get_mut(self.selected);
        if let Some(res) = res {
            res.selected = false
        }
        // Get the new selected item
        let res = self.options.get_mut(new_selected);
        if let Some(res) = res {
            res.selected = true
        }

        self.selected = new_selected;
        self.update_title();
    }

    fn select_next(&mut self) {
        if self.selected >= self.options.len() - 1 {
            return;
        }

        // Check if we have to scroll
        if self.offset + self.max_num_items - 1 == self.selected {
            self.offset += 1;
        }

        let new_selected = self.selected + 1;
        // Get the old selected item
        let res = self.options.get_mut(self.selected);
        if let Some(res) = res {
            res.selected = false;
        }
        // Get the new selected item
        let res = self.options.get_mut(new_selected);
        if let Some(res) = res {
            res.selected = true;
        }
        self.selected = new_selected;
        self.update_title();
    }

    fn update_title(&mut self) {
        self.title = format!(" Options ({}/{}) ", self.selected + 1, self.options.len());
    }

    fn select(&mut self, item_id: usize) {
        if item_id > self.options.len() {
            return;
        }

        let res = self.options.get_mut(item_id);
        if let Some(res) = res {
            res.selected = true;
            self.selected = item_id;
            self.update_title();
        }
    }
}

impl Component for AppOptions {
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
        if mouse.kind == MouseEventKind::Up(MouseButton::Left) {
            self.mouse_click_pos = Some(Position::new(mouse.column, mouse.row));
        }

        Ok(None)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>, CliError> {
        match action {
            Action::NavUp | Action::NavDown => self.kb_select_item(action),
            _ => (),
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<(), CliError> {
        let res = self.check_user_mouse_select(area);
        if let Some(pos) = res {
            self.send_focus_req_action();
            self.mouse_select_item(pos);
        }

        self.render_options_list(frame, area);
        Ok(())
    }
}
