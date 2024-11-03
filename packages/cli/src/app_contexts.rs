use crate::{action::Action, components::theme::ThemeData};

#[derive(Debug)]
pub struct RenderContext<'a> {
    pub modal_open: bool,
    pub theme_data: &'a ThemeData,
}

impl<'a> RenderContext<'a> {
    pub fn new(modal_open: bool, color_data: &'a ThemeData) -> Self {
        Self {
            modal_open,
            theme_data: color_data,
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
