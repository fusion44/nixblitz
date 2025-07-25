use dioxus::prelude::*;

use crate::classes::typography;
use crate::components::Button;

#[component]
pub fn Welcome(
    #[props(optional)] loading: Option<bool>,
    #[props(optional)] on_click: Option<EventHandler<MouseEvent>>,
) -> Element {
    let loading = loading.unwrap_or(false);
    rsx! {
        div { class: "flex flex-col items-center justify-center text-center space-y-8 w-full",
            p { class: "max-w-2xl text-lg text-slate-400",
                "This tool will guide you through installing a NixOS-based system on your computer."
            }

            div { class: "max-w-2xl p-6 rounded-xl border border-red-500 bg-red-900/30",

                div { class: "flex items-center justify-center",
                    span { class: "text-4xl mr-4", "⚠️" }
                    h3 { class: "{typography::headings::H4} text-red-300", "Alpha Software Warning" }
                }

                p { class: "mt-4 text-red-200",
                    "This is an early version of nixblitz. It has not been extensively tested and may contain "
                    strong { class: "font-bold text-red-100", "catastrophic bugs" }
                    " that could lead to "
                    strong { class: "font-bold text-red-100", "complete data loss" }
                    ". Please do not use this on a machine with important data. Proceed at your own risk."
                }
            }
            div {
                Button {
                    loading,
                    on_click: move |evt| {
                        if let Some(callback) = &on_click {
                            callback.call(evt);
                        }
                    },
                    "I Understand, Proceed to Next Step"
                }
            }
        }
    }
}
