use dioxus::prelude::*;
use nixblitz_core::PreInstallConfirmData;

use crate::classes::typography::headings::{H2, H3};
use crate::components::Button;

#[component]
pub fn PreInstallConfirm(
    data: PreInstallConfirmData,
    #[props(optional)] on_confirm: Option<EventHandler<MouseEvent>>,
) -> Element {
    let confirm_button = rsx! {
        div { class: "pt-4 text-center",
            if let Some(on_confirm_handler) = on_confirm {
                {
                    rsx! {
                        Button { on_click: move |evt| on_confirm_handler.call(evt), "Confirm and Start Installation" }
                    }
                }
            }
        }
    };
    rsx! {
        div {
            key: "{data.disk}",
            class: "flex flex-col space-y-6 w-full max-w-2xl mx-auto bg-zinc-900 p-8 rounded-lg border border-zinc-700",

            div { class: "text-center",
                h2 { class: H2, "Installation Summary" }
                p { class: "mt-2 text-md text-slate-400",
                    "Please review the details below before proceeding."
                }
            }

            div { class: "space-y-3",
                h3 { class: H3, "Target Disk" }
                span { class: "text-red-200", "All data on this disk will be erased." }
                div { class: "flex justify-center",
                    span { class: "text-slate-400", "Path:" }
                    div { class: "w-2" }
                    span { class: "font-mono text-slate-200", "{data.disk}" }
                }
            }

            div { class: "space-y-3",
                h3 { class: H3, "Applications to be Installed" }
                if data.apps.is_empty() {
                    p { class: "text-slate-400 italic", "No additional applications selected." }
                } else {
                    ul { class: "list-disc list-inside space-y-1 pl-2 text-slate-300",
                        for app_name in &data.apps {
                            li { key: "{app_name}", "{app_name}" }
                        }
                    }
                }
            }
            {confirm_button}
        }
    }
}
