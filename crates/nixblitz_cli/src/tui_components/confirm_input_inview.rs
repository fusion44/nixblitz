use iocraft::prelude::*;

use crate::tui_components::Confirm;

#[derive(Default, Props)]
pub struct ConfirmInputInViewProps {
    pub value: bool,
    pub on_change: Handler<'static, bool>,
}

#[component]
pub fn ConfirmInputInView<'a>(
    props: &mut ConfirmInputInViewProps,
    hooks: &mut Hooks,
) -> impl Into<AnyElement<'static>> {
    hooks.use_terminal_events({
        let val = props.value;
        let mut on_change = props.on_change.take();
        move |event| match event {
            TerminalEvent::Key(KeyEvent { code, kind, .. }) if kind != KeyEventKind::Release => {
                match code {
                    KeyCode::Char('y') => {
                        on_change(true);
                    }
                    KeyCode::Char('n') => {
                        on_change(false);
                    }
                    KeyCode::Left | KeyCode::Char('h') | KeyCode::Right | KeyCode::Char('l') => {
                        on_change(!val);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    });

    element! {
        Confirm(value: props.value)
    }
}
