use dioxus::prelude::*;
use nixblitz_core::CheckResult;

#[component]
pub fn SystemCheckDisplay(result: CheckResult) -> Element {
    rsx! {
        div {
            h2 { class: "text-xl font-bold mb-2", "System Check Results" }

            if result.is_compatible {
                div { class: "p-2 rounded bg-green-100 text-green-800",
                    "✅ System is compatible and meets all requirements."
                }
            } else {
                div { class: "p-2 rounded bg-red-100 text-red-800",
                    h3 { class: "font-bold", "⚠️ System is not compatible:" }
                    ul { class: "list-disc list-inside",
                        for issue in &result.issues {
                            li { "{issue}" }
                        }
                    }
                }
            }

            div { class: "mt-4",
                h3 { class: "font-semibold", "System Summary:" }
                p { "OS: {result.summary.os_name} ({result.summary.os_version})" }
                p { "Kernel: {result.summary.kernel_version}" }
                p { "Hostname: {result.summary.hostname}" }
                p {
                    "Memory: {result.summary.used_memory / 1024 / 1024} MB / {result.summary.total_memory / 1024 / 1024} MB"
                }
                p { "CPU Cores: {result.summary.cpus.len()}" }
            }
        }
    }
}
