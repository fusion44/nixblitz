use iocraft::prelude::*;
use nixblitz_core::truncate_text;

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
pub fn LogViewer(props: &mut LogViewerProps) -> impl Into<AnyElement<'static>> {
    let height = props.max_height.unwrap_or(props.logs.len() as u16);
    let view_height = (height as usize).saturating_sub(2);
    let displayed_logs = props.logs.iter().rev().take(view_height).rev();

    let lines = displayed_logs.map(|l| {
        let w: Option<usize> = props.width.map(|v| v as usize - 2); // account for borders
        let l = truncate_text(l, None, w);
        element! {
            Text(
                content: l,
                wrap: TextWrap::NoWrap,
            )
        }
    });

    if let Some(width) = props.width {
        element! {
            View (
                border_style: BorderStyle::Round,
                flex_direction: FlexDirection::Column,
                width,
            ) {
                #(lines)
            }
        }
    } else {
        element! {
            View (
                border_style: BorderStyle::Round,
                flex_direction: FlexDirection::Column,
            ) {
                #(lines)
            }
        }
    }
}
