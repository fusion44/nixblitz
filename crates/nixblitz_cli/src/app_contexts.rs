use std::{cell::RefCell, rc::Rc};

use nixblitz_system::project::Project;

use crate::{action::Action, components::theme::ThemeData};

#[derive(Debug)]
pub struct RenderContext {
    pub modal_open: bool,
    pub theme_data: Rc<RefCell<ThemeData>>,
    pub project: Rc<RefCell<Project>>,
}

impl RenderContext {
    pub fn new(
        modal_open: bool,
        theme_data: Rc<RefCell<ThemeData>>,
        project: Rc<RefCell<Project>>,
    ) -> Self {
        Self {
            modal_open,
            theme_data,
            project,
        }
    }
}

#[derive(Debug)]
pub struct UpdateContext {
    pub action: Action,
    pub modal_open: bool,
    pub project: Rc<RefCell<Project>>,
}

impl UpdateContext {
    pub fn new(action: Action, modal_open: bool, project: Rc<RefCell<Project>>) -> Self {
        Self {
            action,
            modal_open,
            project,
        }
    }
}
