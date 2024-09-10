use ratatui::{
    layout::{Direction, Layout, Rect},
    text::Line,
    Frame,
};
use ratatui_macros::constraints;

use crate::errors::CliError;

use super::Component;

#[derive(Debug)]
pub enum OptionType {
    Bool(bool),
    StringVector(Vec<String>),
    SingleLineString(String),
    MultiLineString(String),
    Enum(Vec<String>),
    Password(String),
    UInt(usize),
    UInt8(u8),
    UInt16(u16),
}

impl Default for OptionType {
    fn default() -> Self {
        Self::Bool(false)
    }
}

#[derive(Debug, Default)]
pub struct ListOption {
    pub title: String,
    pub option_type: OptionType,
    pub selected: bool,
}

impl ListOption {
    pub fn new(title: String, option_type: OptionType, selected: bool) -> Self {
        Self {
            title,
            option_type,
            selected,
        }
    }

    pub fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
    }
}

impl Component for ListOption {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<(), CliError> {
        // ╭ Options ────────────────────────────────────────────╮
        // ││T: 21 4                                             │
        // ││s:                                                  │
        // │ T: 21 6                                             │
        // │ s:                                                  │
        // │ T: 21 8                                             │
        // ╰─────────────────────────────────────────────────────╯
        let layout_main = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints![==1, >=1])
            .split(area);

        // [0] decoration at the left -> must be split again
        // [1] vertical layout with title and value
        let layout_options = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints![==1, ==1])
            .split(layout_main[1]);

        let deco_split = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints![==1, ==1])
            .split(layout_main[0]);

        if self.selected {
            frame.render_widget(Line::from("│"), deco_split[0]);
            frame.render_widget(Line::from("│"), deco_split[1]);
        }

        let subtitle = match self.option_type {
            OptionType::Bool(value) => value.to_string(),
            OptionType::StringVector(_) => todo!(),
            OptionType::SingleLineString(_) => todo!(),
            OptionType::MultiLineString(_) => todo!(),
            OptionType::Enum(_) => todo!(),
            OptionType::Password(_) => todo!(),
            OptionType::UInt(_) => todo!(),
            OptionType::UInt8(_) => todo!(),
            OptionType::UInt16(_) => todo!(),
        };

        frame.render_widget(Line::from(self.title.to_owned()), layout_options[0]);
        frame.render_widget(Line::from(subtitle), layout_options[1]);
        Ok(())
    }
}
