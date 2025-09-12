use iocraft::prelude::*;

use crate::tui_components::utils::get_focus_border_color;

#[derive(Default, Props)]
pub struct PopupProps<'a> {
    pub has_focus: bool,
    pub title: String,
    pub children: Vec<AnyElement<'a>>,
    pub spinner: Option<AnyElement<'a>>,
}

#[component]
pub fn Popup<'a>(props: &mut PopupProps<'a>, _hooks: Hooks) -> impl Into<AnyElement<'a>> {
    let title = props.title.to_uppercase();
    let border_color = get_focus_border_color(props.has_focus);

    element! {
        View(
            background_color: Color::Reset,
            border_style: BorderStyle::Round,
            border_color,
            position: Position::Absolute,
            flex_direction: FlexDirection::Column,
        ) {
            View(
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
            ) {
                #(&mut props.spinner)
                Text(content: &title)
            }
            View() {
                #(&mut props.children)
            }
        }
    }
}
