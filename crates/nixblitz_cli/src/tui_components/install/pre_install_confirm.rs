use iocraft::prelude::*;
use nixblitz_core::PreInstallConfirmData;

use crate::tui_components::ConfirmInputInView;

#[derive(Default, Props)]
pub struct PreInstallConfirmProps {
    pub data: PreInstallConfirmData,
    pub on_confirm: Handler<'static, bool>,
}

#[component]
pub fn PreInstallConfirm(
    props: &mut PreInstallConfirmProps,
    hooks: &mut Hooks,
) -> impl Into<AnyElement<'static>> {
    let mut confirmed = hooks.use_state(|| false);

    hooks.use_terminal_events({
        let mut on_confirm = props.on_confirm.take();

        move |event| {
            if let TerminalEvent::Key(KeyEvent {
                code: KeyCode::Enter,
                kind: KeyEventKind::Press,
                ..
            }) = event
            {
                if confirmed.get() {
                    on_confirm(true);
                }
            }
        }
    });

    let confirm_msg = if confirmed.get() {
        "Press Enter to confirm."
    } else {
        "Select YES to confirm and hit Enter to start the installation."
    };
    element! {
        View(flex_direction: FlexDirection::Column, align_items: AlignItems::Center) {
            Text(
                content: "WARNING: All data on the selected disk will be erased!".to_string(),
                color: Color::Red
            )
            Text(content: format!("Disk: {}", props.data.disk))
            View(height: 1)
            ConfirmInputInView(value: confirmed.get(), on_change: move |val| confirmed.set(val))
            Text(content: confirm_msg.to_string())
        }
    }
}
