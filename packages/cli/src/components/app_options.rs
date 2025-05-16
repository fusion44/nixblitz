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

use crossterm::event::{MouseButton, MouseEventKind};
use error_stack::{Report, Result, ResultExt};

use indexmap::IndexMap;
use log::{error, warn};
use nixblitzlib::{
    app_option_data::option_data::{GetOptionId, OptionData},
    project::Project,
};
use ratatui::prelude::*;
use tokio::sync::mpsc::UnboundedSender;

/// Represents a specific UI control for an application option.
///
/// This enum acts as a sum type (or tagged union) to wrap various concrete
/// UI component types, each tailored for a different kind of option data
/// (e.g., boolean, text, number, path). It allows for managing a
/// heterogeneous collection of option controls under a unified type.
///
/// Each variant of `OptionControl` holds a specific component responsible for
/// rendering and handling user input for that particular option type.
enum OptionControl<'a> {
    Bool(BoolOptionComponent),
    StringList(StringListOptionComponent),
    EditText(TextOptionComponent<'a>),
    Path(PathOptionComponent<'a>),
    Password(PasswordOptionComponent<'a>),
    Number(NumberOptionComponent<'a>),
    NetAddress(NetAddressOptionComponent<'a>),
    Port(PortOptionComponent<'a>),
}

impl fmt::Display for OptionControl<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OptionControl::Bool(_) => write!(f, "OptionControl::Bool"),
            OptionControl::StringList(_) => write!(f, "OptionControl::StringList"),
            OptionControl::EditText(_) => write!(f, "OptionControl::EditText"),
            OptionControl::Path(_) => write!(f, "OptionControl::Path"),
            OptionControl::Password(_) => write!(f, "OptionControl::Password"),
            OptionControl::Number(_) => write!(f, "OptionControl::Number"),
            OptionControl::NetAddress(_) => write!(f, "OptionControl::NetAddress"),
            OptionControl::Port(_) => write!(f, "OptionControl::Port"),
        }
    }
}

impl OptionControl<'_> {
    /// Get a mutable reference to the underlying option list item.
    pub fn as_option_list_item_mut(&mut self) -> &mut dyn OptionListItem {
        match self {
            OptionControl::Bool(comp) => comp,
            OptionControl::StringList(comp) => comp,
            OptionControl::EditText(comp) => comp,
            OptionControl::Path(comp) => comp,
            OptionControl::Password(comp) => comp,
            OptionControl::Number(comp) => comp,
            OptionControl::NetAddress(comp) => comp,
            OptionControl::Port(comp) => comp,
        }
    }

    /// Get a mutable reference to the underlying component.
    pub fn as_component_mut(&mut self) -> &mut dyn Component {
        match self {
            OptionControl::Bool(comp) => comp,
            OptionControl::StringList(comp) => comp,
            OptionControl::EditText(comp) => comp,
            OptionControl::Path(comp) => comp,
            OptionControl::Password(comp) => comp,
            OptionControl::Number(comp) => comp,
            OptionControl::NetAddress(comp) => comp,
            OptionControl::Port(comp) => comp,
        }
    }

    /// Update the option component from the provided OptionData instance.
    pub fn update_from_option_data(&mut self, option_data: &OptionData) -> Result<(), CliError> {
        match (self, option_data) {
            (OptionControl::Bool(comp), OptionData::Bool(data)) => comp.set_data(data),
            (OptionControl::StringList(comp), OptionData::StringList(data)) => comp.set_data(data),
            (OptionControl::EditText(comp), OptionData::TextEdit(data)) => comp.set_data(data),
            (OptionControl::Path(comp), OptionData::Path(data)) => comp.set_data(data),
            (OptionControl::Password(comp), OptionData::PasswordEdit(data)) => comp.set_data(data),
            (OptionControl::Number(comp), OptionData::NumberEdit(data)) => comp.set_data(data),
            (OptionControl::NetAddress(comp), OptionData::NetAddress(data)) => comp.set_data(data),
            (OptionControl::Port(comp), OptionData::Port(data)) => comp.set_data(data),
            _ => {
                let data_id = option_data.id().to_string();
                error!(
                    "Mismatched OptionControl variant and OptionData variant for ID: {}",
                    data_id
                );
                return Err(Report::new(CliError::OptionTypeMismatch(
                    "arm unmatched".into(),
                    "unknown".into(),
                )));
            }
        }

        Ok(())
    }
}

#[derive(Default)]
struct OptionMap<'a> {
    map: IndexMap<String, Box<OptionControl<'a>>>,
}

impl<'a> OptionMap<'a> {
    fn new(map: IndexMap<String, Box<OptionControl<'a>>>) -> Self {
        OptionMap { map }
    }

    fn len(&self) -> usize {
        self.map.len()
    }

    fn get_nth_enum_mut(&mut self, index: usize) -> Result<&mut OptionControl<'a>, CliError> {
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

        Ok(option.1.as_component_mut())
    }

    fn get_components_mut(&mut self) -> Result<Vec<&mut dyn Component>, CliError> {
        Ok(self
            .map
            .iter_mut()
            .map(|value| value.1.as_mut().as_component_mut())
            .collect())
    }

    fn get_nth_option_list_item_mut(
        &mut self,
        index: usize,
    ) -> Result<&mut dyn OptionListItem, CliError> {
        let control = self.get_nth_enum_mut(index)?;
        Ok(control.as_option_list_item_mut())
    }
}

#[derive(Default)]
pub struct AppOptions<'a> {
    command_tx: Option<UnboundedSender<Action>>,
    mouse_click_pos: Option<Position>,
    focus: bool,
    options: OptionMap<'a>,
    constraints: Vec<Constraint>,
    selected: usize,
    offset: usize,
    max_num_items: usize,
    title: String,
    modal_open: bool,
    is_even_warning_printed: bool,
}

enum NavDirection {
    Previous,
    Next,
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

        let list_of_options: Result<IndexMap<String, Box<OptionControl>>, CliError> =
            opts.iter()
                .enumerate()
                .map(|(index, option)| {
                    let component: (String, Box<OptionControl<'a>>) =
                        match option {
                            OptionData::Bool(opt) => (
                                opt.id().to_string(),
                                Box::new(OptionControl::Bool(BoolOptionComponent::new(
                                    opt,
                                    index == selected,
                                ))),
                            ),
                            OptionData::StringList(opt) => (
                                opt.id().to_string(),
                                Box::new(OptionControl::StringList(
                                    StringListOptionComponent::new(opt, index == selected),
                                )),
                            ),
                            OptionData::TextEdit(opt) => (
                                opt.id().to_string(),
                                Box::new(OptionControl::EditText(TextOptionComponent::new(
                                    opt,
                                    index == selected,
                                )?)),
                            ),
                            OptionData::Path(opt) => (
                                opt.id().to_string(),
                                Box::new(OptionControl::Path(PathOptionComponent::new(
                                    opt,
                                    index == selected,
                                )?)),
                            ),
                            OptionData::PasswordEdit(opt) => (
                                opt.id().to_string(),
                                Box::new(OptionControl::Password(PasswordOptionComponent::new(
                                    opt,
                                    index == selected,
                                )?)),
                            ),
                            OptionData::NumberEdit(opt) => (
                                opt.id().to_string(),
                                Box::new(OptionControl::Number(NumberOptionComponent::new(
                                    opt,
                                    index == selected,
                                )?)),
                            ),
                            OptionData::NetAddress(opt) => (
                                opt.id().to_string(),
                                Box::new(OptionControl::NetAddress(
                                    NetAddressOptionComponent::new(opt, index == selected)?,
                                )),
                            ),
                            OptionData::Port(opt) => (
                                opt.id().to_string(),
                                Box::new(OptionControl::Port(PortOptionComponent::new(
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

            option_comp.update_from_option_data(option_data)?;
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
            if let Err(e) = tx.send(Action::FocusRequest(FocusableComponent::AppTabOptions)) {
                warn!("Failed to send FocusRequest action: {}", e);
            }
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
        let mut delayed_selected_opt: Option<&mut Box<OptionControl<'_>>> = None;
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
        self.navigate_selection(NavDirection::Previous)
    }

    fn select_next(&mut self) -> Result<(), CliError> {
        self.navigate_selection(NavDirection::Next)
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
        value: &mut OptionControl<'a>,
        frame: &mut Frame<'_>,
        index: Rect,
        ctx: &RenderContext,
    ) -> Result<(), CliError> {
        value.as_component_mut().draw(frame, index, ctx)
    }

    fn navigate_selection(&mut self, direction: NavDirection) -> Result<(), CliError> {
        let options_len = self.options.len();
        if options_len == 0 {
            return Ok(());
        }

        let current_selected_index = self.selected;
        let new_selected_index = match direction {
            NavDirection::Previous => {
                if self.selected == 0 {
                    return Ok(());
                }
                if self.offset > 0 && self.offset == self.selected {
                    self.offset -= 1;
                }
                self.selected - 1
            }
            NavDirection::Next => {
                if self.selected >= options_len - 1 {
                    return Ok(());
                }
                if self.offset + self.max_num_items - 1 == self.selected {
                    self.offset += 1;
                }
                self.selected + 1
            }
        };

        let current_option_control = self
            .options
            .get_nth_enum_mut(current_selected_index)
            .change_context_lazy(|| {
                CliError::GenericError(format!(
                    "Failed to retrieve current option control at presumably valid index {}",
                    current_selected_index
                ))
            })?;
        current_option_control
            .as_option_list_item_mut()
            .set_selected(false);

        let new_option_control = self
            .options
            .get_nth_enum_mut(new_selected_index)
            .change_context_lazy(|| {
                CliError::GenericError(format!(
                    "Failed to retrieve new option control at index {}",
                    new_selected_index
                ))
            })?;
        new_option_control
            .as_option_list_item_mut()
            .set_selected(true);

        self.selected = new_selected_index;
        self.update_title();

        Ok(())
    }
}

impl Component for AppOptions<'_> {
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
                    self.selected = 0;
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
