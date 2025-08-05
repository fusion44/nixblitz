use iocraft::prelude::*;
use nixblitz_core::CheckResult;

#[derive(Default, Props)]
pub struct SystemCheckDisplayProps {
    pub result: CheckResult,
}

#[component]
pub fn SystemCheckDisplay(props: &mut SystemCheckDisplayProps) -> impl Into<AnyElement<'static>> {
    let compatibility_view = if props.result.is_compatible {
        element! {
            View(background_color: Color::Green) {
                Text(
                    content: "✅ System is compatible and meets all requirements.".to_string(),
                    color: Color::Black
                )
            }
        }
        .into_any()
    } else {
        let issues = props
            .result
            .issues
            .iter()
            .map(|issue| {
                element! { Text(content: format!("- {}", issue)) }
            })
            .collect::<Vec<_>>();
        element! {
            View(
                flex_direction: FlexDirection::Column,
                background_color: Color::Red
            ) {
                Text(
                    content: "⚠️ System is not compatible:".to_string(),
                    color: Color::Black
                )
                #(issues)
            }
        }
        .into_any()
    };

    let summary = &props.result.summary;

    element! {
        View(
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexStart,
        ) {
            Text(
                content: "System Check Results".to_string(),
                color: Color::Cyan,
            )
            View(height: 1)
            #(compatibility_view)
            View(height: 1)
            Text(content: "System Summary:".to_string())
            Text(content: format!("OS: {} ({})", summary.os_name, summary.os_version))
            Text(content: format!("Kernel: {}", summary.kernel_version))
            Text(content: format!("Hostname: {}", summary.hostname))
            Text(
                content: format!("Memory: {} MB / {} MB",
                summary.used_memory / 1024 / 1024,
                summary.total_memory / 1024 / 1024)
            )
            Text(content: format!("CPU Cores: {}", summary.cpus.len()))
            View(height: 1)
            Text(content: format!("Press ENTER to configure the system"))
        }
    }
}
