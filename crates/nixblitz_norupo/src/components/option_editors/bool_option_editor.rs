use dioxus::prelude::*;
use dioxus_logger::tracing;
use nixblitz_core::app_option_data::{
    bool_data::BoolOptionChangeData,
    option_data::{OptionDataChangeNotification, OptionId},
};
use tracing::info;

use crate::backend::set_app_option_wrapper;

#[component]
pub(crate) fn BooleanOptionEditor(value: bool, applied: bool, id: OptionId) -> Element {
    let mut value = use_signal(|| value);
    let clone1 = id.clone();

    let set_data = use_callback(move |checked: bool| {
        let clone2 = clone1.clone();
        async move {
            let res =
                set_app_option_wrapper(OptionDataChangeNotification::Bool(BoolOptionChangeData {
                    id: clone2.clone(),
                    value: checked,
                }))
                .await;
        }
    });

    rsx! {
        div { class: "flex items-center",
            label {
                input {
                    class: "form-checkbox h-4 w-4 text-blue-600
                            rounded border-zinc-700 bg-zinc-800 focus:ring-blue-500",
                    r#type: "checkbox",
                    checked: *value.read(),
                    oninput: move |e| set_data.call(e.checked()),
                }
                span { class: "ml-2", "{id.option}" }
            }

            if applied {
                span { class: "text-xs text-yellow-500 ml-2", "(Modified)" }
            }
        }
    }
}
