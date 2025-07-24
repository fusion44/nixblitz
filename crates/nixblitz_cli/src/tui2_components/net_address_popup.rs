use std::net::IpAddr;

use iocraft::prelude::*;

use crate::tui2_components::{CustomTextInput, Popup};

#[derive(Debug, Clone)]
pub enum NetAddressPopupResult {
    Accepted(Option<IpAddr>),
    Cancelled,
}

#[derive(Default, Props)]
pub struct NetAddressPopupProps {
    pub title: &'static str,
    pub text: String,
    pub on_submit: Handler<'static, NetAddressPopupResult>,
}

#[component]
pub fn NetAddressPopup(
    props: &mut NetAddressPopupProps,
    hooks: &mut Hooks,
) -> impl Into<AnyElement<'static>> {
    let mut error_message = hooks.use_state(|| None::<String>);
    let mut current_component = hooks.use_state(|| props.text.clone());

    hooks.use_terminal_events({
        let mut on_submit = props.on_submit.take();

        move |event| match event {
            TerminalEvent::Key(KeyEvent { code, kind, .. }) if kind != KeyEventKind::Release => {
                match code {
                    KeyCode::Enter => {
                        if current_component.read().is_empty() {
                            on_submit(NetAddressPopupResult::Accepted(None));
                            return;
                        }

                        let res = current_component.read().parse::<IpAddr>();
                        if let Ok(ip_addr) = res {
                            on_submit(NetAddressPopupResult::Accepted(Some(ip_addr)));
                            return;
                        }

                        error_message.set(Some("Invalid IP address format".to_string()));
                    }
                    KeyCode::Esc => on_submit(NetAddressPopupResult::Cancelled),
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
            has_focus: true,
            value: current_component.read().clone(),
            on_change: move |new_value| current_component.set(new_value),
        )
    }
    .into_any();

    let mut children = vec![text_input_view];
    if let Some(msg) = msg {
        children.push(msg.into_any());
    }

    element! {
        Popup(
            has_focus: true,
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
