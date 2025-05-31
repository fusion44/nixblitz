use common::app_option_data::{
    bool_data::BoolOptionChangeData,
    option_data::{OptionDataChangeNotification, OptionId},
    password_data::PasswordOptionChangeData,
    text_edit_data::TextOptionChangeData,
};
use dioxus::prelude::*;

use crate::{backend::set_app_option_wrapper, components::input_type::InputType};

#[component]
pub(crate) fn PasswordOptionEditor(value: String, applied: bool, id: OptionId) -> Element {
    let mut value = use_signal(|| value);
    let clone1 = id.clone();

    let set_data = use_callback(move |value: String| {
        let clone2 = clone1.clone();
        async move {
            let res = set_app_option_wrapper(OptionDataChangeNotification::PasswordEdit(
                PasswordOptionChangeData {
                    id: clone2.clone(),
                    value: value.clone(),
                    confirm: Some(value),
                },
            ))
            .await;
        }
    });

    let class  = "p-2 text-sm w-100 rounded-md bg-zinc-800 text-zinc-200 border border-zinc-700 focus:outline-none focus:ring-2 focus:ring-blue-500";

    rsx! {
        div { class: "flex flex-col",
            label {
                input {
                    class,
                    r#type: InputType::Password.as_str(),
                    value: "",
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
