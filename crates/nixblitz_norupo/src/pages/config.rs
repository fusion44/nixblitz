use dioxus::prelude::*;
use dioxus_logger::tracing;
use nixblitz_core::app_option_data::{
    option_data::{GetOptionId, OptionData},
    *,
};

use crate::{
    backend::{get_app_options_wrapper, get_supported_apps_wrapper},
    components::option_editors::*,
};

#[component]
pub fn Config() -> Element {
    let apps = use_resource(get_supported_apps_wrapper);
    let opts = use_resource(move || get_app_options_wrapper("Nix OS".to_string()));
    let mut option_data: Signal<Vec<OptionData>> = use_signal(|| vec![]);

    let mut is_fetching = use_signal(|| false);
    let mut selected_app: Signal<Option<String>> = use_signal(|| Some("Nix OS".to_string()));

    use_effect(move || {
        tracing::trace!("in effect, checking opts resource");
        if let Some(result) = &*opts.read() {
            match result {
                Ok(data) => {
                    tracing::trace!("initial opts data fetched: {:?}", data);
                    *option_data.write() = data.to_vec();
                }
                Err(e) => {
                    tracing::error!("Error fetching initial options data: {}", e);
                }
            }
        }
    });

    let fetch_app_options = use_callback(move |app: String| {
        let mut option_data = option_data.clone();
        let mut is_fetching = is_fetching.clone();
        let mut selected_app = selected_app.clone();

        async move {
            if !*is_fetching.read() {
                *is_fetching.write() = true;
                *selected_app.write() = Some(app.clone());
                *option_data.write() = vec![];

                match get_app_options_wrapper(app).await {
                    Ok(data) => {
                        *option_data.write() = data;
                    }
                    Err(e) => {
                        tracing::error!("Error fetching additional data: {}", e);
                    }
                }
                *is_fetching.write() = false;
            } else {
                tracing::trace!("Fetch already in progress for {}", app);
            }
        }
    });

    let options_list_content: Option<Element> = if *is_fetching.read() {
        Some(rsx! {
            div { class: "text-zinc-400 text-center", "Loading options..." }
        })
    } else if option_data.read().is_empty() {
        Some(rsx! {
            div { class: "text-zinc-400 text-center", "Select an app to load options." }
        })
    } else {
        let opts: Vec<Element> = option_data
            .read()
            .iter()
            .map(|o| match o {
                OptionData::Bool(data) => {
                    rsx! {
                        BooleanOptionEditor {
                            value: data.value(),
                            applied: data.is_applied(),
                            id: data.id().clone(),
                        }
                    }
                }
                OptionData::StringList(data) => {
                    rsx! {
                        StringListOptionEditor {
                            value: data.value(),
                            applied: data.is_applied(),
                            id: data.id().clone(),
                            options: data.options().to_vec(),
                        }
                    }
                }
                OptionData::ManualStringList(data) => {
                    rsx! {
                        ManualStringListOptionEditor {
                            value: data.value().clone(),
                            applied: data.is_applied(),
                            id: data.id().clone(),
                            max_lines: data.max_lines(),
                        }
                    }
                }
                OptionData::TextEdit(data) => {
                    rsx! {
                        TextOptionEditor {
                            value: data.value().to_string(),
                            applied: data.is_applied(),
                            id: data.id().clone(),
                            max_lines: data.max_lines(),
                        }
                    }
                }
                OptionData::Path(data) => {
                    rsx! {
                        PathOptionEditor {
                            value: data.value(),
                            applied: data.is_applied(),
                            id: data.id().clone(),
                        }
                    }
                }
                OptionData::PasswordEdit(data) => {
                    rsx! {
                        PathOptionEditor {
                            value: "",
                            applied: data.is_applied(),
                            id: data.id().clone(),
                        }
                    }
                }
                OptionData::NumberEdit(data) => {
                    rsx! {
                        NumberOptionEditor {
                            value: data.value().clone(),
                            applied: data.is_applied(),
                            id: data.id().clone(),
                        }
                    }
                }
                OptionData::NetAddress(data) => {
                    rsx! {
                        NetAddressOptionEditor {
                            value: data.value(),
                            applied: data.is_applied(),
                            id: data.id().clone(),
                        }
                    }
                }
                OptionData::Port(data) => {
                    rsx! {
                        PortOptionEditor {
                            value: data.value().clone(),
                            applied: data.is_applied(),
                            id: data.id().clone(),
                        }
                    }
                }
                _ => rsx! {
                    div { "Unknown option type" }
                },
            })
            .collect::<Vec<_>>();

        Some(rsx! {
            ul { class: "space-y-1", {opts.into_iter()} }
        })
    };

    let apps_list_content: Option<Element> = match &*apps.read() {
        Some(Ok(apps_vec)) => {
            let mapped_list_items: Vec<Element> = apps_vec
                .iter()
                .map(move |app| {
                    let app_clone = app.clone();
                    let is_selected = selected_app.read().as_ref() == Some(&app);
                    rsx! {
                        li {
                            key: "{app}",
                            class: "list-item p-2 text-sm cursor-pointer transition-colors duration-100 rounded-sm",
                            class: if is_selected { "bg-blue-600 text-white shadow-sm" } else { "bg-transparent text-zinc-200 hover:bg-zinc-700" },
                            onclick: move |_| fetch_app_options.call(app_clone.clone()),
                            "{app}"
                        }
                    }
                })
                .collect();

            Some(rsx! {
                ul { id: "selectable-list", class: "space-y-1", {mapped_list_items.into_iter()} }
            })
        }
        Some(Err(e)) => Some(rsx! {
            div { class: "text-red-500", "Error loading apps: {e}" }
        }),
        None => Some(rsx! {
            div { class: "text-zinc-400", "Loading apps..." }
        }),
    };

    rsx! {
        div { id: "hero",
            div { class: "min-h-screen w-full bg-zinc-950 text-zinc-50 p-4 flex flex-col items-center",
                div { class: "flex flex-col md:flex-row justify-center items-start md:items-stretch w-full max-w-screen-xl space-y-4 md:space-y-0 md:space-x-8",
                    div { class: "p-4 bg-zinc-900 border border-zinc-800 rounded-lg shadow-lg flex flex-col items-center sticky top-4 md:top-8 flex-shrink-0",
                        div { class: "text-sm text-blue-400 mb-2 font-semibold", "Apps" }
                        {apps_list_content}
                    }

                    div { class: "w-full p-4 bg-zinc-900 border border-zinc-800 rounded-lg shadow-lg flex flex-col flex-grow overflow-y-auto h-auto max-h-[calc(100vh-8rem)] md:max-h-[calc(100vh-4rem)]",
                        div { class: "text-sm text-blue-400 mb-2 font-semibold", "Options" }
                        {options_list_content}
                    }
                }
            }
        }
    }
}
