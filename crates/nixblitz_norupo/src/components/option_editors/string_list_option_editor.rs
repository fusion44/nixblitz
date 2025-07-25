use dioxus::prelude::*;
use nixblitz_core::app_option_data::{
    option_data::{OptionDataChangeNotification, OptionId},
    string_list_data::{StringListOptionChangeData, StringListOptionItem},
};

use crate::backend::set_app_option_wrapper;

#[component]
pub(crate) fn StringListOptionEditor(
    value: String,
    applied: bool,
    id: OptionId,
    options: Vec<StringListOptionItem>,
) -> Element {
    let selected_value = use_signal(|| value.clone());

    let clone1 = id.clone();
    let set_data = use_callback(move |value: String| {
        let clone2 = clone1.clone();
        async move {
            let res = set_app_option_wrapper(OptionDataChangeNotification::StringList(
                StringListOptionChangeData {
                    id: clone2.clone(),
                    value,
                },
            ))
            .await;
        }
    });

    rsx! {
        div { class: "flex items-start",
            label {
                select {
                    class: "form-select h-6 w-100 text-zinc-300 rounded border-zinc-700 bg-zinc-800 focus:ring-blue-500",
                    value: "{value}",
                    onchange: move |e| set_data.call(e.value().clone()),
                    for o in options {
                        option { value: "{o.value}", "{o.display_name}" }
                    }
                }
                span { class: "ml-2", "{id.option}" }
            }

            if applied {
                span { class: "text-xs text-yellow-500 mt-1", "(Modified)" }
            }
        }
    }
}
