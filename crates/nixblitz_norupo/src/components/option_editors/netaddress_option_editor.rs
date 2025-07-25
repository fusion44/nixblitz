use std::{net::IpAddr, str::FromStr};

use dioxus::prelude::*;
use dioxus_logger::tracing;
use nixblitz_core::app_option_data::{
    net_address_data::NetAddressOptionChangeData,
    option_data::{OptionDataChangeNotification, OptionId},
};

use crate::backend::set_app_option_wrapper;

#[component]
pub(crate) fn NetAddressOptionEditor(
    value: Option<IpAddr>,
    applied: bool,
    id: OptionId,
) -> Element {
    let value = use_signal(|| {
        if value.is_none() {
            "".to_string()
        } else {
            value.unwrap().to_string()
        }
    });
    let clone1 = id.clone();
    let set_data = use_callback(move |value: String| {
        let addr = if value.is_empty() {
            None
        } else {
            let addr = IpAddr::from_str(&value);
            match addr {
                Ok(v) => Some(v),
                Err(e) => {
                    tracing::error!("Error parsing IP address: {}", e);
                    None
                }
            }
        };

        let clone2 = clone1.clone();
        async move {
            let res = set_app_option_wrapper(OptionDataChangeNotification::NetAddress(
                NetAddressOptionChangeData {
                    id: clone2.clone(),
                    value: Some(addr.unwrap()),
                },
            ))
            .await;
        }
    });

    let class = "p-2 text-sm w-100 rounded-md bg-zinc-800 text-zinc-200 border border-zinc-700 focus:outline-none focus:ring-2 focus:ring-blue-500";

    rsx! {
        div { class: "flex items-start",
            label {
                input {
                    class,
                    value,
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
