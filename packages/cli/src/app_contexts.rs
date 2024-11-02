use crate::action::Action;

#[derive(Debug)]
pub struct RenderContext {
    pub modal_open: bool,
}

impl RenderContext {
    pub fn new(modal_open: bool) -> Self {
        Self { modal_open }
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
