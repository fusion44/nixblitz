use common::app_option_data::{
    bool_data::BoolOptionChangeData,
    option_data::{OptionDataChangeNotification, OptionId},
    path_data::PathOptionChangeData,
    text_edit_data::TextOptionChangeData,
};
use dioxus::prelude::*;

use crate::server::set_app_option_wrapper;

#[component]
pub(crate) fn PathOptionEditor(value: Option<String>, applied: bool, id: OptionId) -> Element {
    let mut value = use_signal(|| {
        if value.is_none() {
            "".to_string()
        } else {
            value.unwrap()
        }
    });
    let clone1 = id.clone();

    let set_data = use_callback(move |value: String| {
        let clone2 = clone1.clone();
        async move {
            let res =
                set_app_option_wrapper(OptionDataChangeNotification::Path(PathOptionChangeData {
                    id: clone2.clone(),
                    value: if value.is_empty() { None } else { Some(value) },
                }))
                .await;
        }
    });

    let class  = "p-2 text-sm w-100 rounded-md bg-zinc-800 text-zinc-200 border border-zinc-700 focus:outline-none focus:ring-2 focus:ring-blue-500";

    rsx! {
        div { class: "flex flex-col",
            label {
                input {
                    class,
                    r#type: "text",
                    value: "{value.read()}",
                    oninput: move |e| set_data.call(e.value().clone()),
                }
                span { class: "ml-2", "{id.option}" }
            }

            if applied {
                span { class: "text-xs text-yellow-500 mt-1", "(Modified)" }
            }
        }
    }
}
