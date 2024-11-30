use std::{cell::RefCell, rc::Rc};

use super::{
    list_options::{
        base_option::OptionListItem, bool::BoolOptionComponent,
        string_list::StringListOptionComponent, text::TextOptionComponent,
    },
    theme::block,
    Component,
};
use crate::{
    action::Action,
    app_contexts::{RenderContext, UpdateContext},
    constants::FocusableComponent,
    errors::CliError,
};
use cli_log::warn;
use crossterm::event::{MouseButton, MouseEventKind};
use error_stack::{Result, ResultExt};

use nixblitzlib::{
    app_option_data::option_data::OptionData, apps::SupportedApps, project::Project,
};
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
    pub fn new(project: Rc<RefCell<Project>>) -> Result<Self, CliError> {
        let opts = Self::get_opts(project, 0, None)?;
        let cons = (0..opts.len()).map(|_| Constraint::Length(2)).collect();
        Ok(Self {
            options: opts,
            constraints: cons,
            ..AppOptions::default()
        })
    }

    fn get_opts(
        project: Rc<RefCell<Project>>,
        selected: usize,
        action_tx: Option<UnboundedSender<Action>>,
    ) -> Result<Vec<Box<dyn OptionListItem>>, CliError> {
        let opts = project
            .clone()
            .borrow_mut()
            .get_app_options()
            .change_context(CliError::Unknown)?;

        Ok(opts
            .iter()
            .enumerate()
            .map(|(index, option)| -> Box<dyn OptionListItem> {
                let mut component: Box<dyn OptionListItem> = match option {
                    OptionData::Bool(opt) => {
                        Box::new(BoolOptionComponent::new(opt, index == selected))
                    }
                    OptionData::StringList(opt) => {
                        Box::new(StringListOptionComponent::new(opt, index == selected))
                    }
                    OptionData::TextEdit(opt) => {
                        Box::new(TextOptionComponent::new(opt, index == selected))
                    }
                };

                if let Some(tx) = action_tx.clone() {
                    let _ = component.register_action_handler(tx);
                }

                component
            })
            .collect())
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

    fn kb_select_item(&mut self, action: &Action) {
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
        ctx: &RenderContext,
    ) -> Result<(), CliError> {
        let block = if ctx.modal_open {
            block::dimmed(&self.title, ctx)
        } else if self.focus {
            block::focused(&self.title, ctx)
        } else {
            block::default(&self.title, ctx)
        };

        let td = ctx.theme_data.clone();
        let block = block
            .bg(td.borrow().colors.surface)
            .fg(td.borrow().colors.on_surface_var);

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

        self.options = Self::get_opts(ctx.project.clone(), self.selected, self.command_tx.clone())?;
        self.constraints = (0..self.options.len())
            .map(|_| Constraint::Length(2))
            .collect();

        for (index, value) in self
            .options
            .iter_mut()
            .skip(self.offset)
            .enumerate()
            .take(self.max_num_items)
        {
            if index == self.selected {
                // defer drawing. The selected option might show a popup,
                // which must be drawn last to make sure it is not overdrawn
                // by options listed later
                continue;
            }
            value.draw(frame, layout[index], ctx)?;
        }
        if let Some(opt) = self.options.get_mut(self.selected) {
            opt.draw(frame, layout[self.selected], ctx)?;
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

    fn handle_key_event(
        &mut self,
        key: crossterm::event::KeyEvent,
    ) -> Result<Option<Action>, CliError> {
        let option = self.options.get_mut(self.selected);
        let option = option.unwrap();
        option.handle_key_event(key)?;

        Ok(None)
    }

    fn update(&mut self, ctx: &UpdateContext) -> Result<Option<Action>, CliError> {
        self.modal_open = ctx.modal_open;

        if !self.modal_open
            && ctx.action != Action::PopModal(false)
            && ctx.action != Action::PopModal(true)
        {
            match ctx.action {
                Action::NavUp | Action::NavDown => self.kb_select_item(&ctx.action),
                Action::Enter => self.on_enter()?,
                _ => (),
            }
        } else {
            let option = self.options.get_mut(self.selected);
            let option = option.unwrap();
            let _ = option.update(ctx);
        }

        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect, ctx: &RenderContext) -> Result<(), CliError> {
        let res = self.check_user_mouse_select(area);
        if let Some(pos) = res {
            self.send_focus_req_action();
            self.mouse_select_item(pos);
        }

        self.render_options_list(frame, area, ctx)?;

        Ok(())
    }
}
