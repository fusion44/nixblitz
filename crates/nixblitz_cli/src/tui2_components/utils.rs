use iocraft::Color;

pub enum NavDirection {
    Previous,
    Next,
}

pub fn get_focus_border_color(has_focus: bool) -> Color {
    if has_focus { Color::Green } else { Color::Blue }
}

pub fn get_selected_char(selected: bool) -> char {
    if selected { '>' } else { ' ' }
}

pub fn get_background_color(selected: bool) -> Color {
    if selected {
        Color::DarkBlue
    } else {
        Color::Reset
    }
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
