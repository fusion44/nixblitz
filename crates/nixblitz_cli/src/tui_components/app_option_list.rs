use std::sync::Arc;

use iocraft::prelude::*;
use nixblitz_core::option_data::OptionData;

use crate::tui_components::{
    SelectableList, SelectableListData, SelectionValue, utils::get_focus_border_color,
};

#[derive(Default, Props)]
pub(crate) struct AppOptionListProps {
    pub has_focus: bool,
    pub height: u16,
    pub width: u16,
    pub on_edit_option: Handler<'static, Option<SelectionValue>>,
    pub options: Arc<Vec<OptionData>>,
}

#[component]
pub(crate) fn AppOptionList(props: &mut AppOptionListProps) -> impl Into<AnyElement<'static>> {
    // move data out of &props leaving a default value in its place
    let data = SelectableListData::Options(props.options.clone());
    let on_edit_option = props.on_edit_option.take();
    element! {
        View(
            width: props.width,
            height: props.height,
            border_style: BorderStyle::Round,
            border_color: get_focus_border_color(props.has_focus),
            justify_content: JustifyContent::Stretch,
            flex_direction: FlexDirection::Column,
        ) {
            View(margin_top: -1) {
                MixedText(
                    align: TextAlign::Center,
                    contents: vec![
                        MixedTextContent::new("<"),
                        MixedTextContent::new("↓↑").color(Color::Green),
                        MixedTextContent::new("> navigate option list "),
                        MixedTextContent::new("<"),
                        MixedTextContent::new("ENTER").color(Color::Green),
                        MixedTextContent::new("> change option"),
                    ]
                )
            }
            SelectableList(
                height: props.height - 2,
                width: props.width - 2,
                has_focus: props.has_focus,
                on_selected: on_edit_option,
                data,
            )
        }

    }
}
