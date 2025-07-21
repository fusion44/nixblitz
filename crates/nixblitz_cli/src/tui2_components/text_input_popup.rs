use iocraft::prelude::*;

use crate::tui2_components::{CustomTextInput, Popup};

#[derive(Debug, Clone)]
pub enum TextInputPopupResult {
    Accepted(String),
    Cancelled,
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
enum PopupFocus {
    #[default]
    Edit,
    Accept,
    Cancel,
}

impl PopupFocus {
    fn next(&self) -> Self {
        match self {
            PopupFocus::Edit => PopupFocus::Accept,
            PopupFocus::Accept => PopupFocus::Cancel,
            PopupFocus::Cancel => PopupFocus::Edit,
        }
    }

    fn previous(&self) -> Self {
        match self {
            PopupFocus::Edit => PopupFocus::Cancel,
            PopupFocus::Accept => PopupFocus::Edit,
            PopupFocus::Cancel => PopupFocus::Accept,
        }
    }
}

#[derive(Default, Props)]
pub struct TextInputPopupProps {
    pub title: &'static str,
    pub max_lines: u16,
    pub text: String,
    pub on_submit: Handler<'static, TextInputPopupResult>,
    pub height: Option<u16>,
}

#[component]
pub fn TextInputPopup(
    props: &mut TextInputPopupProps,
    mut hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let mut focus = hooks.use_state(|| PopupFocus::Edit);
    let mut initial_input_value = hooks.use_state(|| props.text.clone());

    let mut change_focus = move |reverse: bool| {
        let next_focus = match reverse {
            true => focus.read().previous(),
            false => focus.read().next(),
        };
        focus.set(next_focus);
    };

    let is_multiline = props.max_lines > 1;

    hooks.use_terminal_events({
        let mut on_submit = props.on_submit.take();

        move |event| match event {
            TerminalEvent::Key(KeyEvent { code, kind, .. }) if kind != KeyEventKind::Release => {
                match code {
                    KeyCode::Tab => change_focus(false),
                    KeyCode::BackTab => change_focus(true),
                    KeyCode::Enter => match *focus.read() {
                        PopupFocus::Edit => {
                            if !is_multiline {
                                on_submit(TextInputPopupResult::Accepted(
                                    initial_input_value.read().clone(),
                                ));
                            }
                        }
                        PopupFocus::Accept => {
                            on_submit(TextInputPopupResult::Accepted(
                                initial_input_value.read().clone(),
                            ));
                        }
                        PopupFocus::Cancel => {
                            on_submit(TextInputPopupResult::Cancelled);
                        }
                    },
                    KeyCode::Esc => on_submit(TextInputPopupResult::Cancelled),
                    _ => {}
                }
            }
            _ => {}
        }
    });
    let height = if let Some(height) = props.height {
        height
    } else {
        props.max_lines.min(10)
    };
    let text_input_view = element! {
        View(
            height,
            width: 40,
            background_color: Color::DarkGrey,
        ){
            CustomTextInput(
                multiline: is_multiline,
                has_focus: *focus.read() == PopupFocus::Edit,
                value: initial_input_value.read().clone(),
                on_change: move |new_value| initial_input_value.set(new_value),
                masked: false,
            )
        }
    };

    let (buttons_view_height, buttons_view_contents) = if is_multiline {
        let accept_button = element! {
            View(
                height: 1,
                background_color: if *focus.read() == PopupFocus::Accept { Color::Green } else { Color::Reset },
                justify_content: JustifyContent::Center,
            ) {
                Text(content: "ACCEPT", color: Color::White)
            }
        };

        let cancel_button = element! {
            View(
                height: 1,
                background_color: if *focus.read() == PopupFocus::Cancel { Color::Red } else { Color::Reset },
                justify_content: JustifyContent::Center,
            ) {
                Text(content: "CANCEL", color: Color::White)
            }
        };
        (1, vec![accept_button.into_any(), cancel_button.into_any()])
    } else {
        (0, vec![])
    };

    element! {
        Popup(
            has_focus: true,
            title: props.title,
            children: vec![
                element! {
                    View(
                        flex_direction: FlexDirection::Column,
                        border_style: BorderStyle::None,
                        height: (20 + buttons_view_height) as u16,
                        width: 40,
                    ) {
                        #(text_input_view)
                        #(buttons_view_contents)
                    }
                }.into_any()
            ]
        )
    }
}
