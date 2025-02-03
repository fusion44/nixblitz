use core::fmt;
use std::{cell::RefCell, rc::Rc};

use super::{
    list_options::{
        base_option::OptionListItem, bool::BoolOptionComponent,
        net_address::NetAddressOptionComponent, number::NumberOptionComponent,
        password::PasswordOptionComponent, path::PathOptionComponent,
        string_list::StringListOptionComponent, text::TextOptionComponent,
    },
    theme::block,
    Component,
};
use crate::{
    action::Action,
    app_contexts::{RenderContext, UpdateContext},
    components::list_options::port::PortOptionComponent,
    constants::FocusableComponent,
    errors::CliError,
};
use cli_log::{error, warn};
use crossterm::event::{MouseButton, MouseEventKind};
use error_stack::{Report, Result, ResultExt};

use indexmap::IndexMap;
use nixblitzlib::{
    app_option_data::option_data::{GetOptionId, OptionData},
    apps::SupportedApps,
    project::Project,
};
use ratatui::prelude::*;
use tokio::sync::mpsc::UnboundedSender;

enum _Comp<'a> {
    Bool(BoolOptionComponent),
    StringList(StringListOptionComponent),
    EditText(TextOptionComponent<'a>),
    Path(PathOptionComponent<'a>),
    Password(PasswordOptionComponent<'a>),
    Number(NumberOptionComponent<'a>),
    NetAddress(NetAddressOptionComponent<'a>),
    Port(PortOptionComponent<'a>),
}

impl<'a> fmt::Display for _Comp<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            _Comp::Bool(_) => write!(f, "_Comp::Bool"),
            _Comp::StringList(_) => write!(f, "_Comp::StringList"),
            _Comp::EditText(_) => write!(f, "_Comp::EditText"),
            _Comp::Path(_) => write!(f, "_Comp::Path"),
            _Comp::Password(_) => write!(f, "_Comp::Password"),
            _Comp::Number(_) => write!(f, "_Comp::Number"),
            _Comp::NetAddress(_) => write!(f, "_Comp::NetAddress"),
            _Comp::Port(_) => write!(f, "_Comp::Port"),
        }
    }
}

impl<'a> _Comp<'a> {
    fn get_bool_mut(&mut self) -> Result<&mut BoolOptionComponent, CliError> {
        match self {
            _Comp::Bool(ref mut val) => Ok(val),
            _ => Err(Report::new(CliError::OptionTypeMismatch(
                "_Comp::Bool".to_string(),
                format!("{}", self),
            ))),
        }
    }

    fn get_string_list_mut(&mut self) -> Result<&mut StringListOptionComponent, CliError> {
        match self {
            _Comp::StringList(ref mut val) => Ok(val),
            _ => Err(Report::new(CliError::OptionTypeMismatch(
                "_Comp::StringList".to_string(),
                format!("{}", self),
            ))),
        }
    }

    fn get_edit_text_mut(&mut self) -> Result<&mut TextOptionComponent<'a>, CliError> {
        match self {
            _Comp::EditText(ref mut val) => Ok(val),
            _ => Err(Report::new(CliError::OptionTypeMismatch(
                "_Comp::EditText".to_string(),
                format!("{}", self),
            ))),
        }
    }

    fn get_path_mut(&mut self) -> Result<&mut PathOptionComponent<'a>, CliError> {
        match self {
            _Comp::Path(ref mut val) => Ok(val),
            _ => Err(Report::new(CliError::OptionTypeMismatch(
                "_Comp::Path".to_string(),
                format!("{}", self),
            ))),
        }
    }

    fn get_password_mut(&mut self) -> Result<&mut PasswordOptionComponent<'a>, CliError> {
        match self {
            _Comp::Password(ref mut val) => Ok(val),
            _ => Err(Report::new(CliError::OptionTypeMismatch(
                "_Comp::Password".to_string(),
                format!("{}", self),
            ))),
        }
    }

    fn get_number_mut(&mut self) -> Result<&mut NumberOptionComponent<'a>, CliError> {
        match self {
            _Comp::Number(ref mut val) => Ok(val),
            _ => Err(Report::new(CliError::OptionTypeMismatch(
                "_Comp::Number".to_string(),
                format!("{}", self),
            ))),
        }
    }

    fn get_net_address_mut(&mut self) -> Result<&mut NetAddressOptionComponent<'a>, CliError> {
        match self {
            _Comp::NetAddress(ref mut val) => Ok(val),
            _ => Err(Report::new(CliError::OptionTypeMismatch(
                "_Comp::NetAddress".to_string(),
                format!("{}", self),
            ))),
        }
    }

    fn get_port_mut(&mut self) -> Result<&mut PortOptionComponent<'a>, CliError> {
        match self {
            _Comp::Port(ref mut val) => Ok(val),
            _ => Err(Report::new(CliError::OptionTypeMismatch(
                "_Comp::Port".to_string(),
                format!("{}", self),
            ))),
        }
    }

    fn set_selected(&mut self, selected: bool) {
        match self {
            _Comp::Bool(comp) => comp.set_selected(selected),
            _Comp::StringList(comp) => comp.set_selected(selected),
            _Comp::EditText(comp) => comp.set_selected(selected),
            _Comp::Path(comp) => comp.set_selected(selected),
            _Comp::Password(comp) => comp.set_selected(selected),
            _Comp::Number(comp) => comp.set_selected(selected),
            _Comp::NetAddress(comp) => comp.set_selected(selected),
            _Comp::Port(comp) => comp.set_selected(selected),
        }
    }
}

#[derive(Default)]
struct OptionMap<'a> {
    map: IndexMap<String, Box<_Comp<'a>>>,
}

impl<'a> OptionMap<'a> {
    fn new(map: IndexMap<String, Box<_Comp<'a>>>) -> Self {
        OptionMap { map }
    }

    fn len(&self) -> usize {
        self.map.len()
    }

    fn get_nth_enum_mut(&mut self, index: usize) -> Result<&mut _Comp<'a>, CliError> {
        let option = self
            .map
            .iter_mut()
            .nth(index)
            .ok_or(Report::new(CliError::Unknown))?;

        Ok(option.1.as_mut())
    }

    fn get_nth_component_mut(&mut self, index: usize) -> Result<&mut dyn Component, CliError> {
        let option = self
            .map
            .iter_mut()
            .nth(index)
            .ok_or(Report::new(CliError::Unknown))?;

        match option.1.as_mut() {
            _Comp::Bool(bool_option_component) => Ok(bool_option_component),
            _Comp::StringList(string_list_option_component) => Ok(string_list_option_component),
            _Comp::EditText(text_option_component) => Ok(text_option_component),
            _Comp::Path(path_option_component) => Ok(path_option_component),
            _Comp::Password(password_option_component) => Ok(password_option_component),
            _Comp::Number(unum_option_component) => Ok(unum_option_component),
            _Comp::NetAddress(net_address_option_component) => Ok(net_address_option_component),
            _Comp::Port(port_option_component) => Ok(port_option_component),
        }
    }

    fn get_components_mut(&mut self) -> Result<Vec<&mut dyn Component>, CliError> {
        Ok(self
            .map
            .iter_mut()
            .map(|value| match value.1.as_mut() {
                _Comp::Bool(bool_option_component) => bool_option_component as &mut dyn Component,
                _Comp::StringList(string_list_option_component) => string_list_option_component,
                _Comp::EditText(text_option_component) => text_option_component,
                _Comp::Path(path_option_component) => path_option_component,
                _Comp::Password(password_option_component) => password_option_component,
                _Comp::Number(unum_option_component) => unum_option_component,
                _Comp::NetAddress(net_address_option_component) => net_address_option_component,
                _Comp::Port(port_option_component) => port_option_component,
            })
            .collect())
    }

    fn get_nth_option_list_item_mut(
        &mut self,
        index: usize,
    ) -> Result<&mut dyn OptionListItem, CliError> {
        let option = self
            .map
            .iter_mut()
            .nth(index)
            .ok_or(Report::new(CliError::Unknown))?;

        match option.1.as_mut() {
            _Comp::Bool(bool_option_component) => Ok(bool_option_component),
            _Comp::StringList(string_list_option_component) => Ok(string_list_option_component),
            _Comp::EditText(text_option_component) => Ok(text_option_component),
            _Comp::Path(path_option_component) => Ok(path_option_component),
            _Comp::Password(password_option_component) => Ok(password_option_component),
            _Comp::Number(unum_option_component) => Ok(unum_option_component),
            _Comp::NetAddress(net_address_option_component) => Ok(net_address_option_component),
            _Comp::Port(port_option_component) => Ok(port_option_component),
        }
    }
}

#[derive(Default)]
pub struct AppOptions<'a> {
    command_tx: Option<UnboundedSender<Action>>,
    mouse_click_pos: Option<Position>,
    focus: bool,
    options: OptionMap<'a>,
    constraints: Vec<Constraint>,
    app: SupportedApps,
    selected: usize,
    offset: usize,
    max_num_items: usize,
    title: String,
    modal_open: bool,
    is_even_warning_printed: bool,
}

impl<'a> AppOptions<'a> {
    pub fn new(project: Rc<RefCell<Project>>) -> Result<Self, CliError> {
        let opts = Self::build_option_items(project, 0)?;
        let cons = (0..opts.map.len()).map(|_| Constraint::Length(2)).collect();
        Ok(Self {
            options: opts,
            constraints: cons,
            ..AppOptions::default()
        })
    }

    fn build_option_items(
        project: Rc<RefCell<Project>>,
        selected: usize,
    ) -> Result<OptionMap<'a>, CliError> {
        let opts = project
            .borrow_mut()
            .get_app_options()
            .change_context(CliError::Unknown)?;

        let list_of_options: Result<IndexMap<String, Box<_Comp>>, CliError> = opts
            .iter()
            .enumerate()
            .map(|(index, option)| {
                let component: (String, Box<_Comp<'a>>) = match option {
                    OptionData::Bool(opt) => (
                        opt.id().to_string(),
                        Box::new(_Comp::Bool(BoolOptionComponent::new(
                            opt,
                            index == selected,
                        ))),
                    ),
                    OptionData::StringList(opt) => (
                        opt.id().to_string(),
                        Box::new(_Comp::StringList(StringListOptionComponent::new(
                            opt,
                            index == selected,
                        ))),
                    ),
                    OptionData::TextEdit(opt) => (
                        opt.id().to_string(),
                        Box::new(_Comp::EditText(TextOptionComponent::new(
                            opt,
                            index == selected,
                        )?)),
                    ),
                    OptionData::Path(opt) => (
                        opt.id().to_string(),
                        Box::new(_Comp::Path(PathOptionComponent::new(
                            opt,
                            index == selected,
                        )?)),
                    ),
                    OptionData::PasswordEdit(opt) => (
                        opt.id().to_string(),
                        Box::new(_Comp::Password(PasswordOptionComponent::new(
                            opt,
                            index == selected,
                        )?)),
                    ),
                    OptionData::NumberEdit(opt) => (
                        opt.id().to_string(),
                        Box::new(_Comp::Number(NumberOptionComponent::new(
                            opt,
                            index == selected,
                        )?)),
                    ),
                    OptionData::NetAddress(opt) => (
                        opt.id().to_string(),
                        Box::new(_Comp::NetAddress(NetAddressOptionComponent::new(
                            opt,
                            index == selected,
                        )?)),
                    ),
                    OptionData::Port(opt) => (
                        opt.id().to_string(),
                        Box::new(_Comp::Port(PortOptionComponent::new(
                            opt,
                            index == selected,
                        )?)),
                    ),
                };

                Ok(component)
            })
            .collect();

        let list_of_options = list_of_options?;

        Ok(OptionMap::new(list_of_options))
    }

    fn update_option_items(&mut self, project: Rc<RefCell<Project>>) -> Result<(), CliError> {
        let app_option_list = project
            .clone()
            .borrow_mut()
            .get_app_options()
            .change_context(CliError::Unknown)
            .attach_printable("Unable to get app options")?;

        for option_data in app_option_list.iter() {
            let option_id = &option_data.id().to_string();
            let option_comp = self
                .options
                .map
                .get_mut(option_id)
                .ok_or(Report::new(CliError::OptionRetrievalError(
                    option_id.to_string(),
                )))?
                .as_mut();

            match option_data {
                OptionData::Bool(data) => {
                    option_comp.get_bool_mut()?.set_data(data);
                }
                OptionData::StringList(data) => {
                    option_comp.get_string_list_mut()?.set_data(data);
                }
                OptionData::TextEdit(data) => {
                    option_comp.get_edit_text_mut()?.set_data(data);
                }
                OptionData::Path(data) => option_comp.get_path_mut()?.set_data(data),
                OptionData::PasswordEdit(data) => {
                    option_comp.get_password_mut()?.set_data(data);
                }
                OptionData::NumberEdit(data) => {
                    option_comp.get_number_mut()?.set_data(data);
                }
                OptionData::NetAddress(data) => option_comp.get_net_address_mut()?.set_data(data),
                OptionData::Port(data) => {
                    option_comp.get_port_mut()?.set_data(data);
                }
            }
        }

        Ok(())
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

    fn kb_select_item(&mut self, action: &Action) -> Result<(), CliError> {
        match action {
            Action::NavUp => Ok(self.select_previous()?),
            Action::NavDown => Ok(self.select_next()?),
            _ => Ok(()),
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

        let mut delayed_selected_index = 0;
        let mut delayed_selected_opt: Option<&mut Box<_Comp<'_>>> = None;
        for (index, value) in self
            .options
            .map
            .values_mut()
            .skip(self.offset)
            .enumerate()
            .take(self.max_num_items)
        {
            if index == (self.selected - self.offset) {
                // defer drawing. The selected option might show a popup,
                // which must be drawn last to make sure it is not overdrawn
                // by options listed later
                delayed_selected_index = index;
                delayed_selected_opt = Some(value);
                continue;
            }

            Self::draw_opt(value, frame, layout[index], ctx)?;
        }

        if let Some(delayed_selected_opt) = delayed_selected_opt {
            Self::draw_opt(
                delayed_selected_opt,
                frame,
                layout[delayed_selected_index],
                ctx,
            )?;
        }

        Ok(())
    }

    pub fn set_focus(&mut self, focus: bool) {
        self.focus = focus;
    }

    fn select_previous(&mut self) -> Result<(), CliError> {
        if self.selected == 0 {
            self.offset = 0;
            return Ok(());
        }

        // Check if we have to scroll
        if self.offset > 0 && self.offset == self.selected {
            self.offset -= 1;
        }

        let new_selected = self.selected - 1;
        // Get the current selected item and unselect it
        let current_option: &mut _Comp<'a> = self.options.get_nth_enum_mut(self.selected)?;
        current_option.set_selected(false);

        // Get the new selected item and select it
        let new_option = self.options.get_nth_enum_mut(new_selected)?;
        new_option.set_selected(true);

        self.selected = new_selected;
        self.update_title();

        Ok(())
    }

    fn select_next(&mut self) -> Result<(), CliError> {
        if self.selected >= self.options.len() - 1 {
            return Ok(());
        }

        // Check if we have to scroll
        if self.offset + self.max_num_items - 1 == self.selected {
            self.offset += 1;
        }

        let new_selected = self.selected + 1;
        // Get the current selected item and unselect it
        let current_option: &mut _Comp<'a> = self.options.get_nth_enum_mut(self.selected)?;
        current_option.set_selected(false);

        // Get the new selected item and select it
        let new_option = self.options.get_nth_enum_mut(new_selected)?;
        new_option.set_selected(true);

        self.selected = new_selected;
        self.update_title();

        Ok(())
    }

    fn update_title(&mut self) {
        self.title = format!(" Options ({}/{}) ", self.selected + 1, self.options.len());
    }

    pub fn on_enter(&mut self) -> Result<(), CliError> {
        let option = self.options.get_nth_option_list_item_mut(self.selected)?;
        option.on_edit()?;

        Ok(())
    }

    fn draw_opt(
        value: &mut _Comp<'a>,
        frame: &mut Frame<'_>,
        index: Rect,
        ctx: &RenderContext,
    ) -> Result<(), CliError> {
        match value {
            _Comp::Bool(c) => Ok(c.draw(frame, index, ctx)?),
            _Comp::StringList(c) => Ok(c.draw(frame, index, ctx)?),
            _Comp::EditText(c) => Ok(c.draw(frame, index, ctx)?),
            _Comp::Path(c) => Ok(c.draw(frame, index, ctx)?),
            _Comp::Password(c) => Ok(c.draw(frame, index, ctx)?),
            _Comp::Number(c) => Ok(c.draw(frame, index, ctx)?),
            _Comp::NetAddress(c) => Ok(c.draw(frame, index, ctx)?),
            _Comp::Port(c) => Ok(c.draw(frame, index, ctx)?),
        }
    }
}

impl<'a> Component for AppOptions<'a> {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<(), CliError> {
        self.command_tx = Some(tx.clone());
        for c in self.options.get_components_mut()? {
            c.register_action_handler(tx.clone())?;
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
        let option = self.options.get_nth_component_mut(self.selected)?;
        option.handle_key_event(key)?;

        Ok(None)
    }

    fn update(&mut self, ctx: &UpdateContext) -> Result<Option<Action>, CliError> {
        self.modal_open = ctx.modal_open;

        if !self.modal_open
            && ctx.action != Action::PopModal(false)
            && ctx.action != Action::PopModal(true)
            && ctx.action != Action::TogglePasswordVisibility
        {
            match ctx.action {
                Action::NavUp | Action::NavDown => {
                    self.kb_select_item(&ctx.action)?;
                    return Ok(None);
                }
                Action::Enter => {
                    self.on_enter()?;
                    return Ok(None);
                }
                Action::AppTabOptionChangeAccepted => {
                    self.update_option_items(ctx.project.clone())?;
                    return Ok(None);
                }
                Action::AppTabAppSelected(_) => {
                    self.options = Self::build_option_items(ctx.project.clone(), 0)?;
                    self.constraints = (0..self.options.map.len())
                        .map(|_| Constraint::Length(2))
                        .collect();

                    if let Some(tx) = &self.command_tx {
                        for c in self.options.get_components_mut()? {
                            c.register_action_handler(tx.clone())?;
                        }
                    } else {
                        error!("unable to set action sender to new component");
                    }

                    return Ok(None);
                }
                _ => return Ok(None),
            }
        } else {
            let option = self.options.get_nth_component_mut(self.selected)?;
            option.update(ctx)?;
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
