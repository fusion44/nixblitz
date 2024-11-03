use error_stack::{Report, Result};
use ratatui::{
    layout::{Direction, Flex, Layout, Rect},
    style::Stylize,
    widgets::{Block, Clear},
    Frame,
};
use ratatui_macros::constraints;

use crate::{
    app_contexts::RenderContext,
    components::{
        theme::{button, popup},
        Component,
    },
    errors::CliError,
};

#[derive(Debug)]
pub struct PopupConfirmButtonBar {
    state: Option<u16>,
    buttons: Vec<String>,
    button_length: u16,
}

impl PopupConfirmButtonBar {
    pub fn new(state: Option<u16>, buttons: Vec<String>) -> Result<Self, CliError> {
        let button_with_max_length = buttons.iter().max_by_key(|s| s.len()).unwrap().clone();
        if button_with_max_length.len() + 4 > u16::MAX as usize {
            return Err(
                Report::new(CliError::ArgumentError).attach_printable(format!(
                    "Button '{}' violated the max string length of {}",
                    button_with_max_length,
                    u16::MAX
                )),
            );
        }

        Ok(Self {
            state,
            buttons,
            button_length: (button_with_max_length.len() + 4) as u16,
        })
    }
}

impl Component for PopupConfirmButtonBar {
    fn draw(&mut self, frame: &mut Frame, area: Rect, ctx: &RenderContext) -> Result<(), CliError> {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints![==self.button_length,==self.button_length])
            .flex(Flex::SpaceAround)
            .split(area);

        let btn = match self.state {
            Some(v) => v,
            None => u16::MAX,
        };

        frame.render_widget(Clear, area);
        frame.render_widget(
            Block::new().bg(ctx.theme_data.colors.surface_container_high),
            area,
        );
        for (index, button) in self.buttons.iter().enumerate() {
            let p = match btn == index as u16 {
                true => popup::button::focused(button, ctx),
                false => popup::button::default(button, ctx),
            };

            frame.render_widget(p, layout[index]);
        }

        Ok(())
    }
}
