use dioxus::prelude::*;
use dioxus_logger::tracing;
use nixblitz_core::app_option_data::{
    bool_data::BoolOptionChangeData,
    option_data::{OptionDataChangeNotification, OptionId},
    text_edit_data::TextOptionChangeData,
};
use tracing::info;

use crate::backend::set_app_option_wrapper;

#[component]
pub(crate) fn TextOptionEditor(
    value: String,
    applied: bool,
    id: OptionId,
    max_lines: u16,
) -> Element {
    let mut value = use_signal(|| value);
    let clone1 = id.clone();

    let set_data = use_callback(move |value: String| {
        let clone2 = clone1.clone();
        async move {
            let res = set_app_option_wrapper(OptionDataChangeNotification::TextEdit(
                TextOptionChangeData {
                    id: clone2.clone(),
                    value,
                },
            ))
            .await;

            tracing::info!("Result: {:?}", res);
        }
    });

    let max_lines = if max_lines == 1 {
        1
    } else if max_lines > 4 {
        4
    } else {
        max_lines
    };

    let class = "p-2 text-sm w-100 items-start rounded-md bg-zinc-800 text-zinc-200 border border-zinc-700 focus:outline-none focus:ring-2 focus:ring-blue-500";

    rsx! {
        div { class: "flex items-start",
            label {
                if max_lines == 1 {
                    input {
                        class,
                        r#type: "text",
                        value: "{value.read()}",
                        oninput: move |e| set_data.call(e.value().clone()),
                    }
                } else {
                    textarea {
                        class,
                        rows: "{max_lines}",
                        value: "{value.read()}",
                        oninput: move |e| set_data.call(e.value().clone()),
                    }
                }
                span { class: "ml-2", "{id.option}" }
            }

            if applied {
                span { class: "text-xs text-left text-yellow-500 mt-1", "(Modified)" }
            }
        }
    }
}
