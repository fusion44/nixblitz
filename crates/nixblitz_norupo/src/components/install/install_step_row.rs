use dioxus::prelude::*;
use nixblitz_core::{DiskoInstallStep, DiskoStepStatus};

#[component]
pub fn InstallStepRow(step: DiskoInstallStep) -> Element {
    let (icon, icon_class, text_class) = match step.status {
        DiskoStepStatus::Done => ("✅", "text-green-400", "text-slate-200"),
        DiskoStepStatus::InProgress => (
            "",
            "animate-spin rounded-full h-5 w-5 border-2 border-slate-300 border-t-transparent",
            "text-white font-semibold",
        ),
        DiskoStepStatus::Waiting => ("⌛", "text-slate-500", "text-slate-500"),
        DiskoStepStatus::Failed(_) => ("❌", "text-red-500", "text-red-400 font-semibold"),
    };

    rsx! {
        div { class: "flex items-start space-x-4 text-lg",
            div { class: "flex-shrink-0 w-8 h-8 flex items-center justify-center",
                span { class: "{icon_class}", "{icon}" }
            }
            div { class: "flex flex-col items-start",
                span { class: "{text_class}", "{step.name}" }
                if let DiskoStepStatus::Failed(reason) = step.status {
                    p { class: "text-xs text-red-400 mt-1", "{reason}" }
                }
            }
        }
    }
}
