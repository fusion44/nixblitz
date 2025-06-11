use dioxus::prelude::*;
use dioxus_logger::tracing;

use crate::classes::typography;
use crate::components::Button;

#[component]
pub fn Installing(message: String) -> Element {
    rsx! {
        div {
            key: "{message}",
            class: "flex flex-col space-y-6 w-full max-w-2xl mx-auto bg-zinc-900 p-8 rounded-lg border border-zinc-700",
            div { class: "text-center",
                h2 { class: typography::headings::H2, "Installation in progress" }
                p { class: "mt-2 text-md text-slate-400",
                    "Please wait while the installation is in progress."
                }
            }
        }
    }
}
