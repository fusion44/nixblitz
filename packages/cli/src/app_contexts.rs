use std::{cell::RefCell, rc::Rc};

use nixblitzlib::system::System;

use crate::{action::Action, components::theme::ThemeData};

#[derive(Debug)]
pub struct RenderContext {
    pub modal_open: bool,
    pub theme_data: Rc<RefCell<ThemeData>>,
    pub system: Rc<RefCell<System>>,
}

impl RenderContext {
    pub fn new(
        modal_open: bool,
        theme_data: Rc<RefCell<ThemeData>>,
        system: Rc<RefCell<System>>,
    ) -> Self {
        Self {
            modal_open,
            theme_data,
            system,
        }
    }
}

#[derive(Debug)]
pub struct UpdateContext {
    pub action: Action,
    pub modal_open: bool,
}

impl UpdateContext {
    pub fn new(action: Action, modal_open: bool) -> Self {
        Self { action, modal_open }
    }
}
