use ratatui::{
    style::{Color, Style},
    widgets::{Block, Borders},
};

pub fn render_container<'a>(title: &'a str, focused: bool) -> Block<'a> {
    Block::<'a>::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(if focused {
            Color::White
        } else {
            Color::DarkGray
        }))
        .title(title)
        .border_type(ratatui::widgets::BorderType::Rounded)
}
