use iocraft::prelude::*;
use log::info;

use crate::tui_components::{CustomTextInput, Popup};

#[derive(Debug, Clone)]
pub enum TextInputPopupResult {
    Accepted(String),
    Cancelled,
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
enum InternalPopupFocus {
    #[default]
    Edit,
    Accept,
    Cancel,
}

impl InternalPopupFocus {
    fn next(&self) -> Self {
        match self {
            InternalPopupFocus::Edit => InternalPopupFocus::Accept,
            InternalPopupFocus::Accept => InternalPopupFocus::Cancel,
            InternalPopupFocus::Cancel => InternalPopupFocus::Edit,
        }
    }

    fn previous(&self) -> Self {
        match self {
            InternalPopupFocus::Edit => InternalPopupFocus::Cancel,
            InternalPopupFocus::Accept => InternalPopupFocus::Edit,
            InternalPopupFocus::Cancel => InternalPopupFocus::Accept,
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
    pub has_focus: Option<bool>,
}

#[component]
pub fn TextInputPopup(
    props: &mut TextInputPopupProps,
    mut hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    // Holds the internal focus state, which is used to determine the focus from within the popup
    let mut internal_focus = hooks.use_state(|| InternalPopupFocus::Edit);
    let mut initial_input_value = hooks.use_state(|| props.text.clone());
    let has_focus = props.has_focus.unwrap_or(true);

    let mut change_focus = move |reverse: bool| {
        let next_focus = match reverse {
            true => internal_focus.read().previous(),
            false => internal_focus.read().next(),
        };
        internal_focus.set(next_focus);
    };

    let is_multiline = props.max_lines > 1;

    // Calculates the focus state based on the has_focus prop and
    // the internal focus state
    let get_focus = |f: InternalPopupFocus| -> bool {
        if has_focus {
            *internal_focus.read() == f
        } else {
            false
        }
    };

    hooks.use_terminal_events({
        let mut on_submit = props.on_submit.take();
        move |event| match event {
            TerminalEvent::Key(KeyEvent { code, kind, .. }) if kind != KeyEventKind::Release => {
                if !has_focus {
                    return;
                }

                match code {
                    KeyCode::Tab => change_focus(false),
                    KeyCode::BackTab => change_focus(true),
                    KeyCode::Enter => match *internal_focus.read() {
                        InternalPopupFocus::Edit => {
                            if !is_multiline {
                                on_submit(TextInputPopupResult::Accepted(
                                    initial_input_value.read().clone(),
                                ));
                            }
                        }
                        InternalPopupFocus::Accept => {
                            on_submit(TextInputPopupResult::Accepted(
                                initial_input_value.read().clone(),
                            ));
                        }
                        InternalPopupFocus::Cancel => {
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
                has_focus: get_focus(InternalPopupFocus::Edit),
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
                background_color: if *internal_focus.read() == InternalPopupFocus::Accept { Color::Green } else { Color::Reset },
                justify_content: JustifyContent::Center,
            ) {
                Text(content: "ACCEPT", color: Color::White)
            }
        };

        let cancel_button = element! {
            View(
                height: 1,
                background_color: if *internal_focus.read() == InternalPopupFocus::Cancel { Color::Red } else { Color::Reset },
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
            has_focus,
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
