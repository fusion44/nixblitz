use dioxus::prelude::*;
use dioxus_logger::tracing;
use nixblitz_core::{DiskoInstallStep, DiskoStepStatus};

use crate::{classes::typography, components::InstallStepRow};

#[component]
pub fn Installing(
    steps: Signal<Vec<DiskoInstallStep>>,
    logs: Signal<Vec<String>>,
    succeeded: bool,
) -> Element {
    rsx! {
        div { class: "flex flex-col space-y-8 w-full max-w-2xl mx-auto bg-zinc-900 p-8 rounded-lg border border-zinc-700 shadow-2xl",
            div { class: "text-center",
                h2 { class: "{typography::headings::H2} text-slate-100", "Installation in Progress" }
                p { class: "mt-2 text-md text-slate-400",
                    "Your system is being installed. Please do not turn off the power."
                }
            }

            div { class: "w-full border-t border-zinc-700" }

            div { class: "flex flex-col items-start space-y-4 w-full",
                for step in steps.read().iter() {
                    InstallStepRow { key: "{step.name}", step: step.clone() }
                }
            }

            div {
                h3 { class: "text-sm font-semibold text-slate-400 border-b border-zinc-700 pb-2 mb-2",
                    "Detailed Log"
                }
                pre { class: "w-full h-48 bg-black/50 p-4 rounded-md text-xs text-slate-300 font-mono overflow-y-scroll",
                    for log_line in logs.read().iter() {
                        div { "{log_line}" }
                    }
                }
            }
        }
    }
}
