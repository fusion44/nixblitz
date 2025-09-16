use iocraft::prelude::*;

use crate::tui_components::Configurator;

#[derive(Default, Props)]
pub struct ConfiguratorScreenProps {
    pub on_submit: Option<Handler<'static, ()>>,
}

#[component]
pub fn ConfiguratorScreen(
    props: &mut ConfiguratorScreenProps,
    hooks: &mut Hooks,
) -> impl Into<AnyElement<'static>> {
    hooks.use_terminal_events({
        let mut on_submit = props.on_submit.take();
        move |event| {
            if let TerminalEvent::Key(KeyEvent {
                code,
                modifiers,
                kind,
                ..
            }) = event
            {
                if kind != KeyEventKind::Release {
                    match code {
                        KeyCode::Char('a') if modifiers == KeyModifiers::CONTROL => {
                            if let Some(handler) = &mut on_submit {
                                handler(());
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    });

    element! {
        View(
            flex_direction: FlexDirection::Column,
        ) {
            Configurator()
            MixedText(
                align: TextAlign::Center,
                contents: vec![
                    MixedTextContent::new("Press <"),
                    MixedTextContent::new("CTRL + a").color(Color::Green),
                    MixedTextContent::new("> to continue"),
                ]
            )
            MixedText(
                align: TextAlign::Center,
                contents: vec![
                    MixedTextContent::new("Press <"),
                    MixedTextContent::new("q").color(Color::Green),
                    MixedTextContent::new("> to quit"),
                ]
            )
        }
    }
}
