use iocraft::{Color, prelude::*};
use strum::Display;

use crate::tui2_components::{CustomTextInput, Popup, utils::get_text_input_color};

#[derive(Debug, Clone)]
pub enum PasswordInputResult {
    Accepted(String),
    Cancelled,
}

#[derive(Debug, Display, Default, Clone, Copy, PartialEq, Eq)]
pub enum PasswordInputMode {
    #[default]
    GetCurrentPassword,
    SetNewPassword,
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
enum PopupFocus {
    EditPassword,
    #[default]
    EditConfirmPassword,
    Accept,
    Cancel,
}

impl PopupFocus {
    fn next(&self, mode: PasswordInputMode) -> Self {
        match self {
            PopupFocus::EditPassword => {
                if mode == PasswordInputMode::SetNewPassword {
                    PopupFocus::EditConfirmPassword
                } else {
                    PopupFocus::Accept
                }
            }
            PopupFocus::EditConfirmPassword => PopupFocus::Accept,
            PopupFocus::Accept => PopupFocus::Cancel,
            PopupFocus::Cancel => PopupFocus::EditPassword,
        }
    }

    fn previous(&self, mode: PasswordInputMode) -> Self {
        match self {
            PopupFocus::EditPassword => PopupFocus::Cancel,
            PopupFocus::EditConfirmPassword => PopupFocus::EditPassword,
            PopupFocus::Accept => {
                if mode == PasswordInputMode::SetNewPassword {
                    PopupFocus::EditConfirmPassword
                } else {
                    PopupFocus::EditPassword
                }
            }
            PopupFocus::Cancel => PopupFocus::Accept,
        }
    }
}

#[derive(Default, Props)]
pub struct PasswordInputPopupProps {
    pub title: &'static str,
    pub mode: PasswordInputMode,
    pub on_submit: Handler<'static, PasswordInputResult>,
}

#[component]
pub fn PasswordInputPopup(
    props: &mut PasswordInputPopupProps,
    mut hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let mut password_value = hooks.use_state(String::new);
    let mut confirm_password_value = hooks.use_state(String::new);
    let mut focus = hooks.use_state(|| PopupFocus::EditPassword);
    let mut passwords_match = hooks.use_state(|| true);
    let mut hint_text = hooks.use_state(String::new);

    let mode = props.mode;

    let mut change_focus = move |reverse: bool| {
        let next_focus = match reverse {
            true => focus.read().previous(mode),
            false => focus.read().next(mode),
        };
        focus.set(next_focus);
    };

    hooks.use_memo(
        || {
            if mode == PasswordInputMode::SetNewPassword {
                if password_value.read().to_string() == confirm_password_value.read().to_string() {
                    passwords_match.set(true);
                    hint_text.set("match!".to_string());
                } else {
                    passwords_match.set(false);
                    hint_text.set("no match!".to_string());
                }
            }
        },
        (password_value, confirm_password_value),
    );

    hooks.use_terminal_events({
        let mut on_submit = props.on_submit.take();
        let current_password_value = password_value.read().clone();
        let current_confirm_password_value = confirm_password_value.read().clone();

        move |event| match event {
            TerminalEvent::Key(KeyEvent { code, kind, .. }) if kind != KeyEventKind::Release => {
                match code {
                    KeyCode::Tab => change_focus(false),
                    KeyCode::BackTab => change_focus(true),
                    KeyCode::Enter => match *focus.read() {
                        PopupFocus::EditPassword => {
                            if mode == PasswordInputMode::GetCurrentPassword {}
                        }
                        PopupFocus::EditConfirmPassword => {
                            if current_password_value == current_confirm_password_value
                                && !current_password_value.is_empty()
                            {
                                on_submit(PasswordInputResult::Accepted(
                                    current_password_value.clone(),
                                ));
                            } else {
                                hint_text.set("Passwords do not match!".to_string());
                            }
                        }
                        PopupFocus::Accept => match mode {
                            PasswordInputMode::SetNewPassword => {
                                if current_password_value == current_confirm_password_value
                                    && !current_password_value.is_empty()
                                {
                                    hint_text.set("Passwords match!".to_string());
                                    on_submit(PasswordInputResult::Accepted(
                                        current_password_value.clone(),
                                    ));
                                } else {
                                    hint_text.set("Passwords do not match!".to_string());
                                }
                            }
                            PasswordInputMode::GetCurrentPassword => {}
                        },
                        PopupFocus::Cancel => {
                            on_submit(PasswordInputResult::Cancelled);
                        }
                    },
                    KeyCode::Esc => {
                        on_submit(PasswordInputResult::Cancelled);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    });

    let password_input_view = element! {
        View(
            width: 40,
            height: 2,
            flex_direction: FlexDirection::Column,
        ) {
            Text(content: "new password:")
            View(
                height: 1,
                background_color: get_text_input_color(
                    *focus.read() == PopupFocus::EditPassword, false
                )
            ) {
                CustomTextInput(
                    multiline: false,
                    has_focus: *focus.read() == PopupFocus::EditPassword,
                    value: password_value.read().clone(),
                    on_change: move |new_value| {
                        password_value.set(new_value);
                    },
                    masked: true,
                )
            }
        }
    };

    let confirm_password_input_view = if mode == PasswordInputMode::SetNewPassword {
        Some(element! {
            View(
                width: 40,
                flex_direction: FlexDirection::Column,
            ) {
                Text(content: "confirm password:")
                View(
                    height: 1,
                    background_color: get_text_input_color(
                        *focus.read() == PopupFocus::EditConfirmPassword, false
                    )
                ) {
                    CustomTextInput(
                        multiline: false,
                        has_focus: *focus.read() == PopupFocus::EditConfirmPassword,
                        value: confirm_password_value.read().clone(),
                        on_change: move |new_value| {
                            confirm_password_value.set(new_value);
                            hint_text.set(String::new());
                        },
                        masked: true,
                    )
                }
            }
        })
    } else {
        None
    };

    fn button_style(is_focused: bool, is_accept: bool) -> Color {
        if is_focused {
            if is_accept { Color::Green } else { Color::Red }
        } else {
            Color::Reset
        }
    }

    let accept_button = element! {
        View(
            height: 1,
            background_color: button_style(*focus.read() == PopupFocus::Accept, true),
            justify_content: JustifyContent::Center,
        ) {
            Text(content: "ACCEPT", color: Color::White)
        }
    };

    let cancel_button = element! {
        View(
            height: 1,
            background_color: button_style(*focus.read() == PopupFocus::Cancel, false),
            justify_content: JustifyContent::Center,
        ) {
            Text(content: "CANCEL", color: Color::White)
        }
    };

    let mut children = vec![password_input_view.into_any()];

    if let Some(confirm_input) = confirm_password_input_view {
        children.push(confirm_input.into_any());
    }

    let hint_color = if passwords_match.get() {
        Color::Green
    } else {
        Color::Yellow
    };
    children.push(
        element! {
            View(
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                width: 40,
                height: 1,
            ) {
                #(accept_button)
                Text(content: hint_text.read().clone(), color: hint_color)
                #(cancel_button)
            }
        }
        .into_any(),
    );

    element! {
        Popup(
            has_focus: true,
            title: props.title,
            children: vec![
                element! {
                    View(
                        flex_direction: FlexDirection::Column,
                        border_style: BorderStyle::None,
                        height: (
                            if mode == PasswordInputMode::SetNewPassword { 7 } else { 4 }
                        ) as u16,
                        width: 40,
                    ) {
                        #(children)
                    }
                }.into_any()
            ]
        )
    }
}
