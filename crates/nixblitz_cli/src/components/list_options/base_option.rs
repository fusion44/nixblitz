use error_stack::Result;
use ratatui::{
    layout::{Direction, Layout, Rect},
    style::{Color, Modifier, Stylize},
    text::Line,
    Frame,
};
use ratatui_macros::constraints;

use crate::{colors, components::Component, errors::CliError};

pub trait OptionListItem: Component {
    fn selected(&self) -> bool;

    fn set_selected(&mut self, selected: bool);

    fn is_applied(&self) -> bool;

    fn on_edit(&mut self) -> Result<(), CliError>;
}

pub fn draw_item(
    selected: bool,
    title: &str,
    subtitle: &str,
    applied: bool,
    frame: &mut Frame,
    area: Rect,
) -> Result<(), CliError> {
    // ╭ Options ────────────────────────────────────────────╮
    // ││T: 21 4                                             │
    // ││s:                                                  │
    // │ T: 21 6                                             │
    // │ s:                                                  │
    // │ T: 21 8                                             │
    // ╰─────────────────────────────────────────────────────╯
    let layout_main = Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
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

    if selected {
        frame.render_widget(Line::from("│"), deco_split[0]);
        frame.render_widget(Line::from("│"), deco_split[1]);
    }

    // Render the title
    let appendix = if applied { " *" } else { "" };
    let t = format!("{}{}", title, appendix).to_string().to_uppercase();
    let mut modifiers = Modifier::BOLD | Modifier::ITALIC;
    if selected {
        modifiers |= Modifier::REVERSED;
    }
    frame.render_widget(Line::from(t).add_modifier(modifiers), layout_options[0]);

    // Render the subtitle
    if !selected {
        let line = Line::from(subtitle).fg(if applied {
            colors::RED_200
        } else {
            Color::Reset
        });
        frame.render_widget(line, layout_options[1]);
    } else {
        let line = Line::from(subtitle).reversed();
        frame.render_widget(line, layout_options[1]);
    }

    Ok(())
}
