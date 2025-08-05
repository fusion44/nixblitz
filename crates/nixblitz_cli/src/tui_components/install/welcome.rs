use iocraft::prelude::*;

use crate::tui_components::install::Logo;

#[derive(Default, Props)]
pub struct WelcomeProps {
    pub on_click: Option<Handler<'static, ()>>,
    pub connected: bool,
}

#[component]
pub fn Welcome(props: &mut WelcomeProps, hooks: &mut Hooks) -> impl Into<AnyElement<'static>> {
    hooks.use_terminal_events({
        let mut on_click = props.on_click.take();
        move |event| {
            if let TerminalEvent::Key(KeyEvent {
                code: KeyCode::Enter,
                kind: KeyEventKind::Press,
                ..
            }) = event
            {
                if let Some(cb) = on_click.as_deref_mut() {
                    cb(());
                }
            }
        }
    });

    let loading = if props.connected {
        Some(
            element!(Text(content: "Connected to installer engine, press Enter to continue".to_string())),
        )
    } else {
        Some(element!(Text(content: "Connecting to installer engine... please wait".to_string())))
    };

    element! {
        View(
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Stretch
        ) {
            Logo
            View(height: 1) // spacer
            Text(
                content: "Welcome to the NixBlitz Installer!".to_string(),
                color: Color::Cyan,
            )
            #(loading)
        }
    }
}
