use iocraft::prelude::*;
use nixblitz_core::{DiskoInstallStep, DiskoStepStatus};

use crate::tui_components::{LogViewer, Spinner, SpinnerStyle};

#[derive(Default, Props)]
pub struct InstallingProps {
    pub steps: Vec<DiskoInstallStep>,
    pub logs: Vec<String>,
}

#[component]
pub fn Installing(
    props: &mut InstallingProps,
    _hooks: &mut Hooks,
) -> impl Into<AnyElement<'static>> {
    let step_element = props
        .steps
        .iter()
        .map(|step| {
            let step_name = step.name.description_str();
            if step.status == DiskoStepStatus::Done {
                element! { Text(content: format!("[✅] {}", step_name)) }.into_any()
            } else if step.status == DiskoStepStatus::InProgress {
                element! {
                    View(height: 1) {
                        Text(content: "[", weight: Weight::Bold)
                        Spinner(style: SpinnerStyle::BrailleDots)
                        Text(content: format!(" ] {}", step_name), weight: Weight::Bold)
                    }
                }
                .into_any()
            } else {
                element! { Text(content: format!("[⏳] {}", step_name)) }.into_any()
            }
        })
        .collect::<Vec<_>>();

    let logs: Vec<String> = props
        .logs
        .iter()
        .rev()
        .take(8)
        .map(|l| l.to_string())
        .rev()
        .collect();

    element! {
        View(
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center
        ) {
            Text(content: "Installing NixBlitz...".to_string(), color: Color::Cyan)
            View(flex_direction: FlexDirection::Column) {
                #(step_element)
            }
            View(height: 1)
            LogViewer(logs, max_height: 8, width: 80)
        }
    }
}
