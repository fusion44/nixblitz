use dioxus::prelude::*;
use nixblitz_core::{InstallStep, StepStatus};

#[component]
pub fn InstallStepRow(step: InstallStep) -> Element {
    let (icon, icon_class, text_class) = match step.status {
        StepStatus::Done => ("✅", "text-green-400", "text-slate-200"),
        StepStatus::InProgress => (
            "",
            "animate-spin rounded-full h-5 w-5 border-2 border-slate-300 border-t-transparent",
            "text-white font-semibold",
        ),
        StepStatus::Waiting => ("⌛", "text-slate-500", "text-slate-500"),
        StepStatus::Failed(_) => ("❌", "text-red-500", "text-red-400 font-semibold"),
    };

    rsx! {
        div { class: "flex items-start space-x-4 text-lg",
            div { class: "flex-shrink-0 w-8 h-8 flex items-center justify-center",
                span { class: "{icon_class}", "{icon}" }
            }
            div { class: "flex flex-col items-start",
                span { class: "{text_class}", "{step.name}" }
                if let StepStatus::Failed(reason) = step.status {
                    p { class: "text-xs text-red-400 mt-1", "{reason}" }
                }
            }
        }
    }
}
