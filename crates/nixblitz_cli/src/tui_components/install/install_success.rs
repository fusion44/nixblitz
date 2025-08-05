use iocraft::prelude::*;
use log::info;
use nixblitz_core::{DiskoInstallStep, DiskoStepStatus};

use crate::tui_components::Popup;

#[derive(Default, Props)]
pub struct InstallSuccessProps {
    pub final_steps: Vec<DiskoInstallStep>,
    pub final_logs: Vec<String>,
    pub on_reboot: Handler<'static, ()>,
}

#[component]
pub fn InstallSuccess(
    props: &mut InstallSuccessProps,
    hooks: &mut Hooks,
) -> impl Into<AnyElement<'static>> {
    let mut log_popup = hooks.use_state(|| false);

    hooks.use_terminal_events({
        let mut on_reboot = props.on_reboot.take();
        move |event| match event {
            TerminalEvent::Key(KeyEvent {
                code: KeyCode::Enter,
                kind: KeyEventKind::Press,
                ..
            }) => {
                on_reboot(());
            }
            TerminalEvent::Key(KeyEvent {
                code: KeyCode::Char('l'),
                kind: KeyEventKind::Press,
                ..
            }) => {
                log_popup.set(!log_popup.get());
            }
            _ => (),
        }
    });

    let step_element = props
        .final_steps
        .iter()
        .map(|step| {
            if step.status == DiskoStepStatus::Done {
                element! { View() {
                    Text(content: "[✅]")
                    View(width: 1)
                    Text(content: step.name.description_str()) }
                }.into_any()
            } else {
                element! { Text(content: format!("Error: Found a step that is not Done?? {:?}", step)) }
                    .into_any()
            }
        })
        .collect::<Vec<_>>();
    let mut log_elements = props
        .final_logs
        .iter()
        .map(|log| element! { Text(content: log.clone(), color: Color::DarkGrey) })
        .collect::<Vec<_>>();
    if log_elements.is_empty() {
        log_elements
            .push(element! { Text(content: "No logs found.\n(This is a known issue and will be fixed soon™)".to_string()) });
    }

    let popup = if log_popup.get() {
        Some(
            element! {
                Popup(title: "Installation Logs".to_string()) {
                    View(flex_direction: FlexDirection::Column) {
                        #(log_elements)
                    }
                }
            }
            .into_any(),
        )
    } else {
        None
    };

    element! {
        View(flex_direction: FlexDirection::Column, align_items: AlignItems::Stretch) {
            Text(content: "Installation Succeeded!".to_string())
            View(height: 1)
            #(step_element)
            View(height: 1)
            Text(content: "Press Enter to reboot the system.".to_string())
            #(popup)

        }
    }
}
