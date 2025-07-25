use iocraft::prelude::*;

#[derive(Default, Props)]
pub struct ConfirmInputProps<'a> {
    /// The title of the input
    pub title: String,

    /// The value to set when the user confirms the input
    pub value_out: Option<&'a mut bool>, // We have to use an Option to implement Default
}

#[component]
pub fn ConfirmInput<'a>(
    props: &mut ConfirmInputProps<'a>,
    hooks: &mut Hooks,
) -> impl Into<AnyElement<'static>> {
    let mut system = hooks.use_context_mut::<SystemContext>();
    let mut selection = hooks.use_state(|| false);
    let mut should_submit = hooks.use_state(|| false);

    hooks.use_terminal_events({
        move |event| match event {
            TerminalEvent::Key(KeyEvent { code, kind, .. }) if kind != KeyEventKind::Release => {
                match code {
                    KeyCode::Char('y') => {
                        selection.set(true);
                    }
                    KeyCode::Char('n') => {
                        selection.set(false);
                    }
                    KeyCode::Left | KeyCode::Char('h') | KeyCode::Right | KeyCode::Char('l') => {
                        selection.set(!selection.get());
                    }
                    KeyCode::Enter => {
                        should_submit.set(true);
                    }
                    KeyCode::Esc | KeyCode::Char('q') => {
                        selection.set(false);
                        should_submit.set(true);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    });

    if should_submit.get() {
        if let Some(value_out) = props.value_out.as_mut() {
            **value_out = selection.get();
        }
        system.exit();
        element!(View)
    } else {
        element! {
            View(
                flex_direction: FlexDirection::Column,
                height: 2,
            ){
                Text(
                    content: props.title.clone(),
                    align: TextAlign::Center,
                )
                View(
                    flex_direction: FlexDirection::Row,
                    border_style: BorderStyle::None,
                    background_color: Color::Reset,
                ) {
                    View(
                        width: 15,
                        justify_content: JustifyContent::Center,
                        background_color: if selection.get() {
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
                        background_color: if !selection.get() {
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
}
