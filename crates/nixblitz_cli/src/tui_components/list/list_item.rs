use iocraft::prelude::*;
use nixblitz_core::string_list_data::StringListOptionItem;

use crate::tui_components::get_selected_char;

#[derive(Props)]
pub struct ListItemProps {
    pub item: StringListOptionItem,
    pub background_color: iocraft::Color,
    pub prefix: &'static char,
}

impl Default for ListItemProps {
    fn default() -> Self {
        Self {
            item: Default::default(),
            background_color: iocraft::Color::Reset,
            prefix: get_selected_char(false),
        }
    }
}

#[component]
pub fn ListItem(props: &mut ListItemProps, mut _hooks: Hooks) -> impl Into<AnyElement<'static>> {
    element! {
        View(
            flex_direction: FlexDirection::Row,
            background_color: props.background_color,
        ) {
            Text(content: format!("{}{}", props.prefix, props.item.display_name))
        }
    }
}
