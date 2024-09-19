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

    fn is_dirty(&self) -> bool;

    fn on_edit(&mut self) -> Result<(), CliError>;
}

pub fn draw_item(
    selected: bool,
    title: &str,
    subtitle: &str,
    dirty: bool,
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

    let appendix = if dirty { " *" } else { "" };
    let t = format!("{}{}", title, appendix).to_string().to_uppercase();
    frame.render_widget(
        Line::from(t).add_modifier(Modifier::BOLD | Modifier::ITALIC),
        layout_options[0],
    );

    frame.render_widget(
        Line::from(subtitle).fg(if dirty { colors::RED_200 } else { Color::Reset }),
        layout_options[1],
    );

    Ok(())
}
