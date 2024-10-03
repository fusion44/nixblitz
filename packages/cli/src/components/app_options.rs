use super::{
    container::render_container,
    list_options::{
        base_option::OptionListItem,
        bool::BoolOptionComponent,
        string_list::{StringListOption, StringListOptionComponent},
    },
    Component,
};
use crate::{action::Action, constants::FocusableComponent, errors::CliError};
use cli_log::warn;
use crossterm::event::{MouseButton, MouseEventKind};
use error_stack::Result;

use nixblitzlib::{apps::SupportedApps, timezones::TIMEZONES};
use ratatui::prelude::*;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Default)]
pub struct AppOptions {
    command_tx: Option<UnboundedSender<Action>>,
    mouse_click_pos: Option<Position>,
    focus: bool,
    options: Vec<Box<dyn OptionListItem>>,
    constraints: Vec<Constraint>,
    app: SupportedApps,
    selected: usize,
    offset: usize,
    max_num_items: usize,
    title: String,
    modal_open: bool,
    is_even_warning_printed: bool,
}

impl AppOptions {
    pub fn new() -> Self {
        let cons = (0..3).map(|_| Constraint::Length(2)).collect();

        Self {
            options: vec![
                Box::new(BoolOptionComponent::new("bool one", false, true)),
                Box::new(BoolOptionComponent::new("bool two", true, false)),
                Box::new(StringListOptionComponent::new(
                    "system timezone".into(),
                    TIMEZONES[169].to_owned(),
                    false,
                    TIMEZONES
                        .iter()
                        .map(|tz| {
                            StringListOption::new(
                                tz.to_string(),
                                *tz == TIMEZONES[169],
                                tz.to_string(),
                            )
                        })
                        .collect(),
                )),
            ],
            constraints: cons,
            ..AppOptions::default()
        }
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

    fn render_options_list(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        modal_open: bool,
    ) -> Result<(), CliError> {
        let block = render_container(&self.title, if modal_open { false } else { self.focus });
        let total_height = block.inner(area).height;

        let is_even = total_height % 2 == 0;
        if !is_even && !self.is_even_warning_printed {
            self.is_even_warning_printed = true;
            warn!(
            "Area height must be a multiple of two for now.\ntotal_height: {}\nmax_num_items: {}",
            total_height,
            self.max_num_items
        );
        }

        self.max_num_items = (if is_even {
            total_height / 2
        } else {
            (total_height - 1) / 2
        }) as usize;

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
            value.draw(frame, layout[index], modal_open)?;
        }

        Ok(())
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
            res.set_selected(false);
        }
        // Get the new selected item
        let res = self.options.get_mut(new_selected);
        if let Some(res) = res {
            res.set_selected(true);
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
            res.set_selected(false);
        }
        // Get the new selected item
        let res = self.options.get_mut(new_selected);
        if let Some(res) = res {
            res.set_selected(true);
        }
        self.selected = new_selected;
        self.update_title();
    }

    fn update_title(&mut self) {
        self.title = format!(" Options ({}/{}) ", self.selected + 1, self.options.len());
    }

    pub fn on_enter(&mut self) -> Result<(), CliError> {
        let option = self.options.get_mut(self.selected);
        let option = option.unwrap();
        option.on_edit()?;

        Ok(())
    }
}

impl Component for AppOptions {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<(), CliError> {
        self.command_tx = Some(tx.clone());
        for o in &mut self.options {
            o.register_action_handler(tx.clone())?;
        }

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

    fn update(&mut self, action: Action, modal_open: bool) -> Result<Option<Action>, CliError> {
        self.modal_open = modal_open;

        if !modal_open {
            match action {
                Action::NavUp | Action::NavDown => self.kb_select_item(action),
                Action::Enter => self.on_enter()?,
                _ => (),
            }
        } else {
            let option = self.options.get_mut(self.selected);
            let option = option.unwrap();
            let _ = option.update(action.clone(), modal_open);
        }

        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect, modal_open: bool) -> Result<(), CliError> {
        let res = self.check_user_mouse_select(area);
        if let Some(pos) = res {
            self.send_focus_req_action();
            self.mouse_select_item(pos);
        }

        self.render_options_list(frame, area, modal_open)?;

        Ok(())
    }
}
