use iocraft::prelude::*;
use nixblitz_core::string_list_data::StringListOptionItem;

#[derive(Default, Props)]
pub struct ListItemProps {
    pub item: StringListOptionItem,
    pub is_selected: bool,
}

#[component]
pub fn ListItem(props: &mut ListItemProps, mut _hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let background_color = if props.is_selected {
        Color::Blue
    } else {
        Color::Reset
    };

    // Add a prefix to indicate selection
    let prefix = if props.is_selected { "> " } else { "  " };

    element! {
        View(
            flex_direction: FlexDirection::Row,
            background_color,
        ) {
            Text(content: format!("{}{}", prefix, props.item.display_name))
        }
    }
}
