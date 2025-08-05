use iocraft::prelude::*;

#[derive(Default, Props)]
pub struct ConfirmProps {
    pub value: bool,
}

#[component]
pub fn Confirm(props: &mut ConfirmProps) -> impl Into<AnyElement<'static>> {
    element! {
        View(
            flex_direction: FlexDirection::Column,
            height: 2,
        ) {
            View(
                flex_direction: FlexDirection::Row,
                border_style: BorderStyle::None,
                background_color: Color::Reset,
            ) {
                View(
                    width: 15,
                    justify_content: JustifyContent::Center,
                    background_color: if props.value {
                        Color::Green
                    } else {
                        Color::DarkGrey
                    },
                ) {
                    Text(
                        content: "YES".to_string(),
                        color: Color::White,
                        align: TextAlign::Center,
                    )
                }
                Text(content: "  ".to_string(), color: Color::White)
                View(
                    width: 15,
                    justify_content: JustifyContent::Center,
                    background_color: if !props.value {
                        Color::Green
                    } else {
                        Color::DarkGrey
                    },
                ) {
                    Text(
                        content: "NO".to_string(),
                        color: Color::White,
                        align: TextAlign::Center,
                    )
                }
            }
        }
    }
}
