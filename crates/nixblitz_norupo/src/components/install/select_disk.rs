use dioxus::prelude::*;
use nixblitz_core::DiskInfo;

use crate::components::Button;
use crate::string_formatters::format_bytes_to_gb;

#[component]
pub fn InstallDiskSelection(
    disks: Vec<DiskInfo>,
    #[props(optional)] on_select: Option<EventHandler<String>>,
) -> Element {
    let body = if disks.is_empty() {
        vec![rsx! {
            p {
                class: "p-4 text-center text-slate-400",
                "No compatible disks found on your system."
            }
        }]
    } else {
        disks
            .iter()
            .map(|disk| {
                let disk_path_for_closure = disk.path.clone();
                let on_select_handler = on_select;
                let mounts =                         if disk.mount_points.is_empty() {
                            "None".to_string()
                        } else {
                            disk.mount_points.join(", ").to_string()
                        };
                let removable = if disk.is_removable { "Yes" } else { "No" };


                rsx! {
                    div {
                        key: "{disk.path}",
                        class: "grid grid-cols-6 gap-4 items-center p-4 border-t border-zinc-700 text-sm text-slate-200 hover:bg-zinc-800/50",
                        div { class: "truncate text-slate-400 font-mono", "{disk.path}" }
                        div { "{format_bytes_to_gb(disk.size_bytes)}" }
                        div { class: "truncate text-slate-400", {mounts} }
                        div { class: if disk.is_removable { "text-green-400" } else { "text-slate-400" },
                            "{removable}"
                        }
                        div { class: "text-right",
                            Button {
                                on_click: move |_| {
                                    if let Some(callback) = &on_select_handler {
                                        callback.call(disk_path_for_closure.clone());
                                    }
                                },
                                "Select"
                            }
                        }
                    }
                }
            })
            .collect::<Vec<_>>()
    };

    rsx! {
        div { class: "flex flex-col space-y-4 w-full bg-zinc-900 p-6 rounded-lg",

            div { class: "text-center md:text-left",
                h2 { class: "text-2xl font-bold text-white", "Select Installation Disk" }
                p { class: "mt-2 text-md text-slate-400",
                    "Choose the disk where you want to install the system. "
                    strong { class: "text-amber-400", "Warning:" }
                    " The selected disk will be completely erased."
                }
            }

            div { class: "overflow-x-auto rounded-lg border border-zinc-700",
                div { class: "grid grid-cols-6 gap-4 bg-zinc-800 p-4 font-semibold text-left text-sm text-slate-300",
                    div { "Path" }
                    div { "Size" }
                    div { "Mounts" }
                    div { "Removable" }
                    div { class: "text-right", "Action" }
                }

                // --- Table Body ---
                {body.into_iter()}
            }
        }
    }
}
