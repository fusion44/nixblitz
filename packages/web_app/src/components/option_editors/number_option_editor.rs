use common::{
    app_option_data::{
        bool_data::BoolOptionChangeData,
        number_data::NumberOptionChangeData,
        option_data::{OptionDataChangeNotification, OptionId},
        password_data::PasswordOptionChangeData,
        text_edit_data::TextOptionChangeData,
    },
    number_value::NumberValue,
};
use dioxus::prelude::*;

use crate::{backend::set_app_option_wrapper, components::input_type::InputType};

#[component]
pub(crate) fn NumberOptionEditor(value: NumberValue, applied: bool, id: OptionId) -> Element {
    let mut value_str = use_signal(|| value.to_string());
    let clone1 = id.clone();

    let set_data = use_callback(move |new_value: String| {
        // TODO: check if the value is valid
        let v = NumberValue::from_string(new_value, value.clone()).unwrap();
        let clone2 = clone1.clone();
        async move {
            let res = set_app_option_wrapper(OptionDataChangeNotification::Number(
                NumberOptionChangeData {
                    id: clone2.clone(),
                    value: v,
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
                    r#type: InputType::Number.as_str(),
                    value: value_str.read().clone(),
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
