use iocraft::prelude::*;

use crate::tui_components::{
    get_selected_char,
    utils::{get_focus_border_color, get_selected_item_color},
};

#[derive(Default, Props)]
pub(crate) struct AppListProps {
    pub has_focus: bool,
    pub app_list: &'static [&'static str],
    pub selected_item: usize,
    pub height: Option<u16>,
    pub width: Option<u16>,
}

#[component]
pub(crate) fn AppList(
    props: &mut AppListProps,
    mut hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let (width, _) = hooks.use_terminal_size();
    let height = if let Some(h) = props.height {
        h
    } else {
        props.app_list.len() as u16 + 2
    };

    let selected = props.selected_item;
    let items = props.app_list.iter().enumerate().map(|(i, app)| {
        let char = get_selected_char(i == selected);
        let background_color = get_selected_item_color(i == selected, props.has_focus);
        let max_text_width = props.width.unwrap_or(width).saturating_sub(4);
        let display_text = if app.len() > max_text_width as usize {
            format!(
                "{} {}...",
                char,
                &app[..max_text_width.saturating_sub(4) as usize]
            )
        } else {
            format!("{} {}", char, app)
        };

        element! {
            View(background_color: background_color) {
                Text(content: display_text.to_uppercase(), color: Color::White)
            }
        }
    });

    let border_color = get_focus_border_color(props.has_focus);
    if let Some(width) = props.width {
        element! {
            View(
                height,
                width,
                flex_direction: FlexDirection::Column,
                border_style: BorderStyle::Round,
                border_color,
            ) {
                #(items)
            }
        }
    } else {
        element! {
            View(
                height,
                flex_direction: FlexDirection::Column,
                border_style: BorderStyle::Round,
                border_color,
            ) {
                #(items)
            }
        }
    }
}
