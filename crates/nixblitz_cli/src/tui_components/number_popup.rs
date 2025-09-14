use iocraft::prelude::*;
use nixblitz_core::NumberValue;

use crate::tui_components::{CustomTextInput, Popup};

#[derive(Debug, Clone)]
pub enum NumberPopupResult {
    Accepted(NumberValue),
    Cancelled,
}

#[derive(Default, Props)]
pub struct NumberPopupProps {
    pub title: &'static str,
    pub value: NumberValue,
    pub on_submit: Handler<'static, NumberPopupResult>,
    pub has_focus: Option<bool>,
}

#[component]
pub fn NumberPopup(
    props: &mut NumberPopupProps,
    hooks: &mut Hooks,
) -> impl Into<AnyElement<'static>> {
    let has_focus = props.has_focus.unwrap_or(true);
    let mut error_message = hooks.use_state(|| None::<String>);
    let mut current_value = hooks.use_state(|| props.value.to_string_or(""));

    hooks.use_terminal_events({
        let mut on_submit = props.on_submit.take();
        let template_value = props.value.clone();

        move |event| match event {
            TerminalEvent::Key(KeyEvent { code, kind, .. }) if kind != KeyEventKind::Release => {
                if !has_focus {
                    return;
                }

                match code {
                    KeyCode::Enter => {
                        let input_str = current_value.read();
                        let optional_input = if input_str.is_empty() {
                            None
                        } else {
                            Some(input_str.as_str())
                        };

                        match template_value.parse_as_variant(optional_input) {
                            Ok(parsed_value) => {
                                on_submit(NumberPopupResult::Accepted(parsed_value));
                            }
                            Err(_) => {
                                let hint = match &template_value {
                                    NumberValue::U16(_) => {
                                        format!("Please enter a number between 0 and {}.", u16::MAX)
                                    }
                                    NumberValue::UInt(_) => {
                                        "Please enter a positive whole number.".to_string()
                                    }
                                    NumberValue::Int(_) => {
                                        "Please enter a whole number (e.g., -100).".to_string()
                                    }
                                    NumberValue::Float(_) => {
                                        "Please enter a number (e.g., 3.14).".to_string()
                                    }
                                };
                                error_message.set(Some(hint));
                            }
                        }
                    }
                    KeyCode::Esc => on_submit(NumberPopupResult::Cancelled),
                    _ => {}
                }
            }
            _ => {}
        }
    });

    let msg = error_message
        .read()
        .as_ref()
        .map(|msg| element! {Text(content: msg.clone(), color: Color::Red)});

    let text_input_view = element! {
        CustomTextInput(
            has_focus,
            value: current_value.read().clone(),
            on_change: move |new_value| current_value.set(new_value),
        )
    }
    .into_any();

    let mut children = vec![text_input_view];
    if let Some(msg) = msg {
        children.push(msg.into_any());
    }

    element! {
        Popup(
            has_focus,
            title: props.title,
            children: vec![
                element! {
                    View(
                        flex_direction: FlexDirection::Column,
                        border_style: BorderStyle::None,
                        width: 40,
                    ) {
                        #(children)
                    }
                }.into_any()
            ]
        )
    }
}
