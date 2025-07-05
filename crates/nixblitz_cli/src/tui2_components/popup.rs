use iocraft::prelude::*;

#[derive(Default, Props)]
pub struct PopupProps<'a> {
    pub has_focus: bool,
    pub title: String,
    pub children: Vec<AnyElement<'a>>,
}

#[component]
pub fn Popup<'a>(props: &mut PopupProps<'a>, _hooks: Hooks) -> impl Into<AnyElement<'a>> {
    let title = props.title.to_uppercase();

    element! {
        View(
            background_color: Color::Reset,
            border_style: BorderStyle::Round,
            border_color: if props.has_focus { Color::Green } else { Color::Reset },
            position: Position::Absolute,
            flex_direction: FlexDirection::Column,
        ) {
            View(
                background_color: Color::Blue,
                justify_content: JustifyContent::Center,
            ) {
                Text(content: &title)
            }
            View() {
                #(&mut props.children)
            }
        }
    }
}
