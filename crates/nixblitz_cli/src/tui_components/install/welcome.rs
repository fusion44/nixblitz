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
        Some(element! {
            View(
                flex_direction: FlexDirection::Column,
            ) {
                View(
                    width: 60,
                    flex_direction: FlexDirection::Column,
                    border_style: BorderStyle::Round,
                    border_color: Color::Red,
                ) {
                    Text(content: "Alpha Software Warning",
                         color: Color::Red,
                         align: TextAlign::Center
                    )
                    Text(content: "This is an early version of nixblitz. It has not been extensively tested and may contain catastrophic bugs that could lead to complete data loss. Do not use this on a machine with important data. Proceed at your own risk.")
                }
                Text(
                    content: "Connected to installer engine",
                    align: TextAlign::Center,
                    color: Color::Cyan
                )
                MixedText(
                    align: TextAlign::Center,
                    contents: vec![
                        MixedTextContent::new("Press <"),
                        MixedTextContent::new("ENTER").color(Color::Green),
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

        }.into_any())
    } else {
        Some(
            element! {
                Text(content: "Connecting to installer engine... please wait".to_string())
            }
            .into_any(),
        )
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
