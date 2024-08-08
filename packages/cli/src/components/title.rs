use color_eyre::Result;
use ratatui::{prelude::*, widgets::*};

use super::Component;

#[derive(Debug, Default)]
pub struct Title {
    pub title: String,
}

impl Title {
    pub fn new(title: String) -> Self {
        Self { title }
    }
}

impl Component for Title {
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        frame.render_widget(Paragraph::new(self.title.clone()), area);
        Ok(())
    }
}
