use iocraft::prelude::*;
use nixblitz_core::truncate_text;
use std::cmp::min;

#[derive(Default, Props)]
pub struct LogViewerProps {
    /// The log lines to display.
    pub logs: Vec<String>,
    /// The maximum height for the log view area.
    pub max_height: Option<u16>,
    /// The width of the log view area.
    /// If not provided, the width will be determined automatically.
    pub width: Option<u16>,
}

#[component]
pub fn LogViewer(props: &mut LogViewerProps, hooks: &mut Hooks) -> impl Into<AnyElement<'static>> {
    let available_width = hooks.use_terminal_size().0;
    let width = props.width.map(|w| w.min(available_width));
    let height = props
        .max_height
        .unwrap_or_else(|| min(props.logs.len(), u16::MAX as usize) as u16);

    let view_height = (height as usize).saturating_sub(2);
    let displayed_logs = props.logs.iter().rev().take(view_height).rev();
    let truncation_width = width.map(|v| (v as usize).saturating_sub(2));

    let lines = displayed_logs.map(|log_line| {
        let truncated_line = truncate_text(log_line, None, truncation_width);
        element! {
            Text(
                content: truncated_line,
                wrap: TextWrap::NoWrap,
            )
        }
    });

    if let Some(w) = width {
        element! {
            View (
                border_style: BorderStyle::Round,
                flex_direction: FlexDirection::Column,
                width: w,
                height,
            ) {
                #(lines)
            }
        }
    } else {
        element! {
            View (
                border_style: BorderStyle::Round,
                flex_direction: FlexDirection::Column,
                height,
            ) {
                #(lines)
            }
        }
    }
}
