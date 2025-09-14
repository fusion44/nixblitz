use iocraft::prelude::*;

use crate::tui_components::Popup;

#[derive(Default, Props)]
pub struct EngineOffHelpPopupProps {
    pub on_confirm: Handler<'static, ()>,
    pub has_focus: Option<bool>,
}

#[component]
pub fn EngineOffHelpPopup(
    props: &mut EngineOffHelpPopupProps,
    mut hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let has_focus = props.has_focus.unwrap_or(true);
    let text = element! {
        MixedText(
            align: TextAlign::Center,
            contents: vec![
                MixedTextContent::new("The system engine is not running. "),
                MixedTextContent::new("Changes will not be applied automatically.\n"),
                MixedTextContent::new("Please exit the TUI and run "),
                MixedTextContent::new("'sudo nixblitz apply'").color(Color::Green),
                MixedTextContent::new(" to apply your changes.")
            ]
        )
    };

    let btn_color = if has_focus {
        Color::Green
    } else {
        Color::Reset
    };

    let ok_button = element! {
        View(
            height: 1,
            width: 10,
            justify_content: JustifyContent::Center,
            background_color: btn_color,
        ) {
            Text(content: "OK".to_string())
        }
    };

    hooks.use_terminal_events({
        let mut on_confirm = props.on_confirm.take();
        move |event| match event {
            TerminalEvent::Key(KeyEvent { code, kind, .. }) if kind != KeyEventKind::Release => {
                match code {
                    KeyCode::Enter => on_confirm(()),
                    KeyCode::Esc => on_confirm(()),
                    _ => {}
                }
            }
            _ => {}
        }
    });

    element! {
        Popup(
            has_focus: true,
            title: "System Engine Offline".to_string(),
            children: vec![
                element! {
                    View(
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center
                    ) {
                        #(text)
                        View(height:1)
                        #(ok_button)
                    }
                }.into_any()
            ]
        )
    }
    .into_any()
}
