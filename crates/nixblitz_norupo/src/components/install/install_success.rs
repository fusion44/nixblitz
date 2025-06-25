use dioxus::prelude::*;
use dioxus_logger::tracing;
use nixblitz_core::{DiskoInstallStep, DiskoStepStatus};

use crate::{
    classes::typography,
    components::{Button, InstallStepRow},
};

#[component]
pub fn InstallSuccess(steps: Vec<DiskoInstallStep>, on_reboot: EventHandler<()>) -> Element {
    let steps = steps.iter().map(|step| {
        rsx! {
            InstallStepRow { key: "{step.name}", step: step.clone() }
        }
    });
    rsx! {
        div { class: "flex flex-col space-y-8 w-full max-w-2xl mx-auto bg-zinc-900 p-8 rounded-lg border border-green-500 shadow-2xl",

            div { class: "text-center",
                div { class: "text-6xl text-green-400 mb-4", "✔" }
                h2 { class: "{typography::headings::H2} text-slate-100", "Installation Successful" }
                p { class: "mt-2 text-md text-slate-400",
                    "Your system has been installed correctly. You may now reboot into your new system."
                }
            }

            div { class: "w-full border-t border-zinc-700" }

            div { class: "flex flex-col items-start space-y-4 w-full", {steps} }

            div { class: "pt-6 border-t border-zinc-700 flex justify-center items-center space-x-4" }

            Button { on_click: move |_| on_reboot.call(()), "Reboot Now" }
        }
    }
}
