use crate::{app_contexts::RenderContext, errors::CliError};
use error_stack::Result;

use ratatui::{prelude::*, widgets::*};
use ratatui_macros::constraint;
use tui_textarea::TextArea;

use super::{Component, theme::input};

#[derive(Debug, Default)]
pub struct PasswordInput<'a> {
    focused: bool,
    show_password: bool,
    text_area: TextArea<'a>,
    focus_indicator: bool,
}

impl PasswordInput<'_> {
    pub fn new(
        placeholder: Option<&'static str>,
        focused: bool,
        show_password: bool,
        focus_indicator: bool,
    ) -> Result<Self, CliError> {
        let mut ta = TextArea::default();
        if let Some(placeholder) = placeholder {
            ta.set_placeholder_text(placeholder);
        }

        let mut instance = Self {
            text_area: ta,
            ..Default::default()
        };
        instance.set_focused(focused);
        instance.set_show_password(show_password);
        instance.set_focus_indicator(focus_indicator);

        Ok(instance)
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    pub fn set_show_password(&mut self, show: bool) {
        self.show_password = show;
        if !self.show_password {
            self.text_area.set_mask_char('\u{2022}');
        } else {
            self.text_area.clear_mask_char();
        }
    }

    fn set_focus_indicator(&mut self, focus_indicator: bool) {
        self.focus_indicator = focus_indicator;
    }

    pub fn input(&mut self, key: crossterm::event::KeyEvent) -> bool {
        self.text_area.input(key)
    }

    pub fn lines(&self) -> &[std::string::String] {
        self.text_area.lines()
    }
}

impl Component for PasswordInput<'_> {
    fn draw(&mut self, frame: &mut Frame, area: Rect, ctx: &RenderContext) -> Result<(), CliError> {
        let (indicator_area, text_area) = if self.focus_indicator {
            let l = Layout::default()
                .direction(ratatui::layout::Direction::Horizontal)
                .constraints([constraint!(==2), constraint!(>=1)])
                .split(area);
            (Some(l[0]), l[1])
        } else {
            (None, area)
        };

        if self.focused {
            self.text_area.set_style(input::focused(ctx));
            self.text_area.set_cursor_style(input::cursor::focused(ctx));
            if let Some(indicator_area) = indicator_area {
                frame.render_widget(
                    Paragraph::new(Span::styled(">", Style::default())),
                    indicator_area,
                );
            }
        } else {
            self.text_area.set_style(input::default(ctx));
            self.text_area.set_cursor_style(input::cursor::default(ctx));
        }

        frame.render_widget(&self.text_area, text_area);

        Ok(())
    }
}
