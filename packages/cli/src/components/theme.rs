use std::{fmt::Debug, str::FromStr};

use error_stack::{Report, Result, ResultExt};
use log::debug;
use ratatui::style::Color;

use serde_json::{error::Category, Value};

use crate::{components::default_theme::DEFAULT_THEME_NO_TRUE_COLOR, errors::CliError};

use super::default_theme::DEFAULT_COLOR_THEME;

// Create your own theme:
// https://material-foundation.github.io/material-theme-builder/

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Colors {
    // primary colors
    pub primary: Color,
    pub on_primary: Color,
    pub primary_container: Color,
    pub on_primary_container: Color,
    pub primary_inverse: Color,

    // secondary colors
    pub secondary: Color,
    pub on_secondary: Color,
    pub secondary_container: Color,
    pub on_secondary_container: Color,

    // tertiary colors
    pub tertiary: Color,
    pub on_tertiary: Color,
    pub tertiary_container: Color,
    pub on_tertiary_container: Color,

    // error colors
    pub error: Color,
    pub on_error: Color,
    pub error_container: Color,
    pub on_error_container: Color,

    // surface colors
    pub surface: Color,
    pub surface_var: Color,
    pub surface_tint: Color,
    pub surface_inverse: Color,
    pub surface_dim: Color,
    pub surface_bright: Color,
    pub surface_container_lowest: Color,
    pub surface_container_low: Color,
    pub surface_container: Color,
    pub surface_container_high: Color,
    pub surface_container_highest: Color,
    pub on_surface: Color,
    pub on_surface_inverse: Color,
    pub on_surface_var: Color,

    // outline colors
    pub outline: Color,
    pub outline_var: Color,

    // other
    pub scrim: Color,
    pub shadow: Color,
}

impl Default for Colors {
    fn default() -> Self {
        Self {
            // primary colors
            primary: Color::from_str("#b1d18a").unwrap(),
            on_primary: Color::from_str("#1f3701").unwrap(),
            primary_container: Color::from_str("#354e16").unwrap(),
            on_primary_container: Color::from_str("#cdeda3").unwrap(),
            primary_inverse: Color::from_str("#4C662B").unwrap(),

            // secondary colors
            secondary: Color::from_str("#bfcbad").unwrap(),
            on_secondary: Color::from_str("#2a331e").unwrap(),
            secondary_container: Color::from_str("#404a33").unwrap(),
            on_secondary_container: Color::from_str("#dce7c8").unwrap(),

            // tertiary colors
            tertiary: Color::from_str("#a0d0cb").unwrap(),
            on_tertiary: Color::from_str("#003735").unwrap(),
            tertiary_container: Color::from_str("#1f4e4b").unwrap(),
            on_tertiary_container: Color::from_str("#bcece7").unwrap(),

            // error colors
            error: Color::from_str("#ffb4ab").unwrap(),
            on_error: Color::from_str("#690005").unwrap(),
            error_container: Color::from_str("#93000a").unwrap(),
            on_error_container: Color::from_str("#ffdad6").unwrap(),

            // surface colors
            surface: Color::from_str("#12140e").unwrap(),
            surface_var: Color::from_str("#44483d").unwrap(),
            surface_tint: Color::from_str("#b1d18a").unwrap(),
            surface_inverse: Color::from_str("#e2e3d8").unwrap(),
            surface_dim: Color::from_str("#12140e").unwrap(),
            surface_bright: Color::from_str("#383a32").unwrap(),
            surface_container_lowest: Color::from_str("#0c0f09").unwrap(),
            surface_container_low: Color::from_str("#1a1c16").unwrap(),
            surface_container: Color::from_str("#1e201a").unwrap(),
            surface_container_high: Color::from_str("#282b24").unwrap(),
            surface_container_highest: Color::from_str("#33362e").unwrap(),
            on_surface: Color::from_str("#e2e3d8").unwrap(),
            on_surface_inverse: Color::from_str("#2f312a").unwrap(),
            on_surface_var: Color::from_str("#c5c8ba").unwrap(),

            // outline colors
            outline: Color::from_str("#8f9285").unwrap(),
            outline_var: Color::from_str("#44483d").unwrap(),

            // other
            scrim: Color::from_str("#000000").unwrap(),
            shadow: Color::from_str("#000000").unwrap(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ThemeData {
    pub theme_name: String,
    pub theme_scheme: String,
    pub colors: Colors,
}

impl Default for ThemeData {
    fn default() -> Self {
        Self {
            theme_name: "pale-green".into(),
            theme_scheme: "dark".into(),
            colors: Default::default(),
        }
    }
}

impl ThemeData {
    pub fn set_theme(&mut self, name: &str, scheme: &str) -> Result<(), CliError> {
        self.theme_name = name.to_string();
        self.theme_scheme = scheme.to_string();
        debug!("{}", DEFAULT_COLOR_THEME);
        self.colors = self.load_theme(DEFAULT_THEME_NO_TRUE_COLOR)?;
        Ok(())
    }

    fn load_theme(&self, theme_data: &str) -> Result<Colors, CliError> {
        let json_value: Value = Self::parse_json(theme_data, &self.theme_scheme)?;
        let theme_colors = Colors {
            // Parse primary colors
            primary: Self::parse_color(&json_value, "primary")?,
            on_primary: Self::parse_color(&json_value, "onPrimary")?,
            primary_container: Self::parse_color(&json_value, "primaryContainer")?,
            on_primary_container: Self::parse_color(&json_value, "onPrimaryContainer")?,

            // Parse secondary colors
            secondary: Self::parse_color(&json_value, "secondary")?,
            on_secondary: Self::parse_color(&json_value, "onSecondary")?,
            secondary_container: Self::parse_color(&json_value, "secondaryContainer")?,
            on_secondary_container: Self::parse_color(&json_value, "onSecondaryContainer")?,

            // Parse tertiary colors
            tertiary: Self::parse_color(&json_value, "tertiary")?,
            on_tertiary: Self::parse_color(&json_value, "onTertiary")?,
            tertiary_container: Self::parse_color(&json_value, "tertiaryContainer")?,
            on_tertiary_container: Self::parse_color(&json_value, "onTertiaryContainer")?,

            // Parse error colors
            error: Self::parse_color(&json_value, "error")?,
            on_error: Self::parse_color(&json_value, "onError")?,
            error_container: Self::parse_color(&json_value, "errorContainer")?,
            on_error_container: Self::parse_color(&json_value, "onErrorContainer")?,

            // Parse surface colors
            surface: Self::parse_color(&json_value, "surface")?,
            surface_var: Self::parse_color(&json_value, "surfaceVariant")?,
            surface_tint: Self::parse_color(&json_value, "surfaceTint")?,
            surface_inverse: Self::parse_color(&json_value, "inverseSurface")?,
            surface_dim: Self::parse_color(&json_value, "surfaceDim")?,
            surface_bright: Self::parse_color(&json_value, "surfaceBright")?,
            surface_container_lowest: Self::parse_color(&json_value, "surfaceContainerLowest")?,
            surface_container_low: Self::parse_color(&json_value, "surfaceContainerLow")?,
            surface_container: Self::parse_color(&json_value, "surfaceContainer")?,
            surface_container_high: Self::parse_color(&json_value, "surfaceContainerHigh")?,
            surface_container_highest: Self::parse_color(&json_value, "surfaceContainerHighest")?,
            on_surface_var: Self::parse_color(&json_value, "onSurfaceVariant")?,
            outline: Self::parse_color(&json_value, "outline")?,
            outline_var: Self::parse_color(&json_value, "outlineVariant")?,
            shadow: Self::parse_color(&json_value, "shadow")?,
            scrim: Self::parse_color(&json_value, "scrim")?,
            on_surface_inverse: Self::parse_color(&json_value, "inverseOnSurface")?,
            primary_inverse: Self::parse_color(&json_value, "inversePrimary")?,
            on_surface: Self::parse_color(&json_value, "onSurface")?,
        };

        Ok(theme_colors)
    }

    fn parse_json(theme_data: &str, theme_scheme: &str) -> Result<Value, CliError> {
        let json_value: Value = serde_json::from_str(theme_data).map_err(|e| {
            let category = match e.classify() {
                Category::Io => "I/O error",
                Category::Syntax => "Syntax error",
                Category::Data => "Data error",
                Category::Eof => "Unexpected EOF",
            };
            let error_message = format!("{}: {}", category, e);
            Report::new(CliError::JsonParseError).attach_printable(error_message)
        })?;

        let json_value = json_value["schemes"][theme_scheme].clone();

        Ok(json_value)
    }

    fn parse_color(json_value: &serde_json::Value, key: &str) -> Result<Color, CliError> {
        Color::from_str(json_value[key].as_str().unwrap()).change_context(CliError::Unknown)
    }
}

pub mod block {
    use ratatui::{
        style::Style,
        widgets::{Block, BorderType, Borders},
    };

    use crate::app_contexts::RenderContext;

    pub fn default<'a>(title: &'a str, ctx: &RenderContext) -> Block<'a> {
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(
                Style::new()
                    .bg(ctx.theme_data.clone().borrow().colors.surface)
                    .fg(ctx.theme_data.clone().borrow().colors.on_surface),
            )
    }

    pub fn focused<'a>(title: &'a str, ctx: &RenderContext) -> Block<'a> {
        default(title, ctx).border_style(
            Style::new()
                .bg(ctx.theme_data.clone().borrow().colors.surface)
                .fg(ctx.theme_data.clone().borrow().colors.primary),
        )
    }

    pub fn dimmed<'a>(title: &'a str, ctx: &RenderContext) -> Block<'a> {
        default(title, ctx).border_style(
            Style::new()
                .bg(ctx.theme_data.clone().borrow().colors.surface_dim)
                .fg(ctx.theme_data.clone().borrow().colors.on_surface_var),
        )
    }

    pub fn no_border(_: &RenderContext) -> Block {
        Block::default().borders(Borders::NONE)
    }
}

pub mod button {

    use ratatui::{
        style::{Style, Stylize},
        text::Span,
        widgets::Paragraph,
    };

    use crate::app_contexts::RenderContext;

    pub fn default<'a>(label: &'a str, ctx: &RenderContext) -> Paragraph<'a> {
        let style = Style::default()
            .bg(ctx.theme_data.clone().borrow().colors.surface)
            .fg(ctx.theme_data.clone().borrow().colors.primary);
        Paragraph::new(Span::styled(label, style)).centered()
    }

    pub fn focused<'a>(label: &'a str, ctx: &RenderContext) -> Paragraph<'a> {
        let style = Style::default()
            .bg(ctx.theme_data.clone().borrow().colors.surface)
            .fg(ctx.theme_data.clone().borrow().colors.secondary);
        Paragraph::new(Span::styled(label, style)).centered().bold()
    }
}

pub mod menu {
    use ratatui::{
        style::{Style, Stylize},
        text::{Line, Span},
        widgets::Tabs,
    };

    use crate::app_contexts::RenderContext;

    pub fn item<'a>(title: &'a str, action_char_index: usize, ctx: &RenderContext) -> Line<'a> {
        let (a, b) = title.split_at(action_char_index);
        if action_char_index == 1 {
            Line::from(vec![
                Span::styled(
                    a,
                    Style::default()
                        .fg(ctx.theme_data.clone().borrow().colors.primary)
                        .underlined(),
                ),
                Span::styled(
                    b,
                    Style::default().fg(ctx.theme_data.clone().borrow().colors.on_surface),
                ),
            ])
            .bg(ctx.theme_data.clone().borrow().colors.surface)
        } else {
            let (c, d) = a.split_at(a.len() - 1);
            Line::from(vec![
                Span::styled(
                    c,
                    Style::default().fg(ctx.theme_data.clone().borrow().colors.on_surface),
                ),
                Span::styled(
                    d,
                    Style::default()
                        .fg(ctx.theme_data.clone().borrow().colors.primary)
                        .underlined(),
                ),
                Span::styled(
                    b,
                    Style::default().fg(ctx.theme_data.clone().borrow().colors.on_surface),
                ),
            ])
            .bg(ctx.theme_data.clone().borrow().colors.surface)
        }
    }

    pub fn tab_bar<'a>(items: Vec<Line<'a>>, active_item: usize, ctx: &RenderContext) -> Tabs<'a> {
        Tabs::new(items)
            .select(active_item)
            .divider(Span::raw("|"))
            .highlight_style(
                Style::default()
                    .bold()
                    .fg(ctx.theme_data.clone().borrow().colors.primary),
            )
            .bg(ctx.theme_data.clone().borrow().colors.surface)
            .fg(ctx.theme_data.clone().borrow().colors.on_surface)
    }
}
pub mod popup {
    use ratatui::{
        style::{Style, Stylize},
        widgets::{Block, Borders, Padding},
    };

    use crate::app_contexts::RenderContext;

    /// A block as should be used in popups
    pub fn block<'a>(title: String, ctx: &RenderContext) -> Block<'a> {
        Block::default()
            .bg(ctx
                .theme_data
                .clone()
                .borrow()
                .colors
                .surface_container_high)
            .borders(Borders::ALL)
            .title(title)
            .title_alignment(ratatui::layout::Alignment::Center)
            .padding(Padding::horizontal(1))
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(Style::new().fg(ctx.theme_data.clone().borrow().colors.primary_inverse))
    }

    pub fn block_focused<'a>(title: String, ctx: &RenderContext) -> Block<'a> {
        block(title, ctx)
            .border_style(Style::new().fg(ctx.theme_data.clone().borrow().colors.primary))
    }

    pub mod button {
        use ratatui::{
            style::{Style, Stylize},
            text::Span,
            widgets::Paragraph,
        };

        use crate::app_contexts::RenderContext;

        pub fn default<'a>(label: &'a str, ctx: &RenderContext) -> Paragraph<'a> {
            let style = Style::default()
                .bg(ctx
                    .theme_data
                    .clone()
                    .borrow()
                    .colors
                    .surface_container_high)
                .fg(ctx.theme_data.clone().borrow().colors.primary);
            Paragraph::new(Span::styled(label, style)).centered()
        }

        pub fn focused<'a>(label: &'a str, ctx: &RenderContext) -> Paragraph<'a> {
            let style = Style::default()
                .bg(ctx
                    .theme_data
                    .clone()
                    .borrow()
                    .colors
                    .surface_container_high)
                .fg(ctx.theme_data.clone().borrow().colors.secondary);
            Paragraph::new(Span::styled(label, style)).centered().bold()
        }
    }

    pub mod error_text {
        use ratatui::{style::Style, text::Span, widgets::Paragraph};

        use crate::app_contexts::RenderContext;

        pub fn default<'a>(label: &'a str, ctx: &RenderContext) -> Paragraph<'a> {
            let style = Style::default()
                .bg(ctx.theme_data.clone().borrow().colors.error_container)
                .fg(ctx.theme_data.clone().borrow().colors.error);
            Paragraph::new(Span::styled(label, style))
        }
    }
}

pub mod input {
    use ratatui::style::Style;

    use crate::app_contexts::RenderContext;

    /// Input style, not focused
    pub fn default(ctx: &RenderContext) -> Style {
        Style::default().fg(ctx.theme_data.borrow().colors.on_surface)
    }

    /// Input style, focused
    pub fn focused(ctx: &RenderContext) -> Style {
        Style::default().fg(ctx.theme_data.borrow().colors.on_surface_var)
    }

    pub mod cursor {
        use ratatui::style::Style;

        use crate::app_contexts::RenderContext;

        pub fn default(ctx: &RenderContext) -> Style {
            Style::default().bg(ctx.theme_data.borrow().colors.surface_container_high)
        }

        pub fn focused(ctx: &RenderContext) -> Style {
            Style::default().bg(ctx.theme_data.borrow().colors.primary)
        }
    }
}

pub mod list {
    use ratatui::{
        style::{Modifier, Style, Stylize},
        text::Line,
        widgets::{List, ListItem},
    };

    use crate::{app_contexts::RenderContext, colors};

    use super::block;

    /// Represents an item within a Popup menu.
    #[derive(Debug)]
    pub struct SelectableListItem {
        /// The underlying value associated with the item.
        pub value: String,

        /// Indicates whether the item is currently selected.
        pub selected: bool,

        /// The title displayed for the item in the Popup menu.
        pub display_title: String,
    }

    impl From<&SelectableListItem> for ListItem<'_> {
        fn from(value: &SelectableListItem) -> Self {
            let line = match value.selected {
                false => Line::styled(format!(" ☐ {}", value.display_title), colors::WHITE),
                true => Line::styled(format!(" ✓ {}", value.display_title), colors::CYAN_500),
            };
            ListItem::new(line)
        }
    }
    fn build<'a>(items: &'a [&'a str], ctx: &RenderContext) -> List<'a> {
        List::new(items.to_owned())
            .bg(ctx.theme_data.clone().borrow().colors.surface)
            .fg(ctx.theme_data.clone().borrow().colors.on_surface_var)
            .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
            .highlight_symbol(">")
            .repeat_highlight_symbol(true)
    }

    pub fn default<'a>(title: &'a str, items: &'a [&'a str], ctx: &RenderContext) -> List<'a> {
        build(items, ctx).block(block::default(title, ctx))
    }

    pub fn focused<'a>(title: &'a str, items: &'a [&'a str], ctx: &RenderContext) -> List<'a> {
        build(items, ctx).block(block::focused(title, ctx))
    }

    pub fn dimmed<'a>(title: &'a str, items: &'a [&'a str], ctx: &RenderContext) -> List<'a> {
        build(items, ctx).block(block::dimmed(title, ctx))
    }

    /// A list that allows items to be selected
    pub mod select {
        use ratatui::{
            style::{Modifier, Style},
            widgets::{List, ListItem},
        };

        use crate::app_contexts::RenderContext;

        use super::SelectableListItem;

        pub fn default<'a>(items: &[SelectableListItem], _: &RenderContext) -> List<'a> {
            let list_items: Vec<ListItem> = items.iter().map(ListItem::from).collect();
            List::new(list_items)
                .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
                .highlight_symbol(">")
                .repeat_highlight_symbol(true)
        }
    }
}
