use dioxus::prelude::*;

use crate::classes::buttons;

#[component]
pub fn Button(
    children: Element,
    #[props(optional)] loading: Option<bool>,
    #[props(optional)] on_click: Option<EventHandler<MouseEvent>>,
) -> Element {
    let is_loading = loading.unwrap_or(false);
    let children = if is_loading {
        rsx! {
            div { class: "animate-spin rounded-full h-5 w-5 border-2 border-slate-300 border-t-transparent" }
        }
    } else {
        rsx! {
            {children}
        }
    };

    rsx! {
        button {
            class: buttons::DEFAULT,
            disabled: is_loading,
            onclick: move |evt| {
                if let Some(callback) = &on_click {
                    callback.call(evt);
                }
            },
            {children}
        }
    }
}
