use iocraft::prelude::*;
use nixblitz_core::{DiskoInstallStep, DiskoStepStatus};

use crate::tui_components::{Spinner, SpinnerStyle};

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

    let log_elements = props
        .logs
        .iter()
        .rev()
        .take(3)
        .map(|log| element! { Text(content: log.clone(), color: Color::DarkGrey) })
        .collect::<Vec<_>>();

    element! {
        View(flex_direction: FlexDirection::Column) {
            Text(content: "Installing NixBlitz...".to_string(), color: Color::Cyan)
            View(flex_direction: FlexDirection::Column) {
                #(step_element)
            }
            View(height: 1)
            View(flex_direction: FlexDirection::Column) {
                Text(content: "==== Last 3 logs entries ====".to_string())
                #(log_elements)
            }
        }
    }
}
