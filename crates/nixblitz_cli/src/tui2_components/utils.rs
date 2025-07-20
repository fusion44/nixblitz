use iocraft::{AnyElement, Color};

pub enum NavDirection {
    Previous,
    Next,
}

pub fn get_focus_border_color(has_focus: bool) -> Color {
    if has_focus { Color::Green } else { Color::Grey }
}

pub fn get_selected_item_color(selected: bool, component_focused: bool) -> Color {
    if selected && component_focused {
        Color::Blue
    } else if selected && !component_focused {
        Color::DarkGrey
    } else {
        get_background_color()
    }
}

pub fn get_selected_char(selected: bool) -> &'static char {
    if selected { &'>' } else { &' ' }
}

pub fn get_background_color() -> Color {
    Color::Reset
}

pub fn format_bool_subtitle(value: bool) -> String {
    if value {
        return "✓ (true)".to_string();
    }

    "✗ (false)".to_string()
}

pub struct NavSelectionResult {
    pub selected: usize,
    pub offset: usize,
}

impl NavSelectionResult {
    fn new(selected: usize, offset: usize) -> Self {
        Self { selected, offset }
    }
}

pub fn navigate_selection(
    direction: NavDirection,
    selected: usize,
    offset: usize,
    options_len: usize,
    max_num_items: usize,
) -> NavSelectionResult {
    if options_len == 0 {
        return NavSelectionResult::new(0, 0);
    }

    match direction {
        NavDirection::Previous => {
            if selected == 0 && options_len <= max_num_items {
                NavSelectionResult::new(options_len - 1, 0)
            } else if selected == 0 && options_len > max_num_items {
                NavSelectionResult::new(options_len - max_num_items, options_len - max_num_items)
            } else if offset > 0 && offset == selected {
                NavSelectionResult::new(selected - 1, offset - 1)
            } else {
                NavSelectionResult::new(selected - 1, offset)
            }
        }
        NavDirection::Next => {
            if selected == options_len - 1 {
                NavSelectionResult::new(0, 0)
            } else if offset + max_num_items - 1 == selected {
                NavSelectionResult::new(selected + 1, offset + 1)
            } else {
                NavSelectionResult::new(selected + 1, offset)
            }
        }
    }
}

/// Generic item trait that can be implemented by different data types
pub trait SelectableItem: Clone {
    type SelectionValue: Clone;

    fn item_height(&self) -> u16;

    // We have four states:
    // 1. The component is focused and the item is not selected
    // 2. The component is focused and the item is selected
    // 3. The component is not focused and the item is not selected
    // 4. The component is not focused and the item is selected
    fn render(
        &self,
        is_selected: bool,
        component_focused: bool,
        width: Option<u16>,
    ) -> AnyElement<'static>;
}
