use std::sync::Arc;

use iocraft::prelude::*;
use log::error;
use nixblitz_core::{
    OPTION_TITLES,
    option_data::{GetOptionId, OptionData, OptionId},
    string_list_data::StringListOptionItem,
    truncate_text,
};

use crate::tui_components::{
    ListItem, NavDirection, get_selected_char, navigate_selection,
    utils::{SelectableItem, format_bool_subtitle, get_selected_item_color},
};

impl SelectableItem for StringListOptionItem {
    type SelectionValue = usize;

    fn item_height(&self) -> u16 {
        1
    }

    fn render(
        &self,
        is_selected: bool,
        component_focused: bool,
        _: Option<u16>,
    ) -> AnyElement<'static> {
        let background_color = get_selected_item_color(is_selected, component_focused);
        let prefix = get_selected_char(is_selected);
        element! {
            ListItem(
                item: self.clone(),
                background_color,
                prefix,
             )
        }
        .into_any()
    }
}

impl SelectableItem for OptionData {
    type SelectionValue = OptionId;

    fn item_height(&self) -> u16 {
        2
    }

    fn render(
        &self,
        is_selected: bool,
        component_focused: bool,
        width: Option<u16>,
    ) -> AnyElement<'static> {
        let char = get_selected_char(is_selected);
        let background_color = get_selected_item_color(is_selected, component_focused);
        let id = self.id();
        let title = OPTION_TITLES.get(id).map_or("title_not_found", |v| v);

        let subtitle = match self {
            OptionData::Bool(b) => format_bool_subtitle(b.value()),
            OptionData::StringList(s) => s.value().to_string(),
            OptionData::ManualStringList(m) => m.value().join(", "),
            OptionData::TextEdit(t) => t.first_line().to_string(),
            OptionData::Path(p) => p.value().unwrap_or_default(),
            OptionData::PasswordEdit(_) => "********".to_string(),
            OptionData::NumberEdit(n) => n.value().to_string(),
            OptionData::NetAddress(n) => n.value().map_or(String::new(), |v| v.to_string()),
            OptionData::Port(p) => p.value().to_string(),
        };

        let available_width = width.map_or(usize::MAX, |w| w.saturating_sub(4) as usize);

        let char_str = &char.to_string();
        let title_text = truncate_text(title, Some(char_str), Some(available_width)).to_uppercase();
        let subtitle_text = truncate_text(&subtitle, Some(char_str), Some(available_width));
        if let Some(width) = width {
            element! {
                View(width, flex_direction: FlexDirection::Column, background_color) {
                    Text(content: title_text, color: Color::White, wrap: TextWrap::NoWrap)
                    Text(content: subtitle_text)
                }
            }
        } else {
            element! {
                View(flex_direction: FlexDirection::Column, background_color) {
                    Text(content: title_text, color: Color::White, wrap: TextWrap::NoWrap)
                    Text(content: subtitle_text)
                }
            }
        }
        .into_any()
    }
}

#[derive(Clone)]
pub enum SelectableListData {
    Options(Arc<Vec<OptionData>>),
    StringListItems(Vec<StringListOptionItem>),
}

impl Default for SelectableListData {
    fn default() -> Self {
        SelectableListData::StringListItems(Vec::new())
    }
}

impl SelectableListData {
    fn len(&self) -> usize {
        match self {
            SelectableListData::Options(options) => options.len(),
            SelectableListData::StringListItems(items) => items.len(),
        }
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[derive(Debug, Clone)]
pub enum SelectionValue {
    OptionId(OptionId),
    Index(usize),
}

#[derive(Default, Props)]
pub struct SelectableListProps {
    pub has_focus: bool,
    pub on_selected: Handler<'static, Option<SelectionValue>>,
    pub data: SelectableListData,
    pub debug_info: bool,
    pub width: Option<u16>,
    pub height: Option<u16>,
}

#[component]
pub fn SelectableList(
    props: &mut SelectableListProps,
    mut hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    if props.data.is_empty() {
        error!("No items given to SelectableList");
        return element! {
            View {
                Text(content: "ERROR: No items available")
            }
        };
    }

    let num_items = props.data.len();
    let item_height = match &props.data {
        SelectableListData::Options(o) => o.first().map_or(2, |item| item.item_height()),
        SelectableListData::StringListItems(i) => i.first().map_or(1, |item| item.item_height()),
    } as usize;

    let available_height = if let Some(h) = props.height {
        h
    } else {
        // If no height is provided, we expect that we have enough space to render the entire list.
        num_items as u16
    };
    let max_num_list_items = available_height as usize / item_height;

    let mut selected = hooks.use_state(|| 0);
    let mut offset = hooks.use_state(|| 0);

    hooks.use_terminal_events({
        let mut on_selected = props.on_selected.take();
        let data = props.data.clone();

        let focus = props.has_focus;
        move |event| {
            // Note: We always have to register the use_terminal_events hook,
            //       even if the component is not focused. We MUST to return here
            //       to avoid the terminal events being handled by the component
            if !focus {
                return;
            }

            if let TerminalEvent::Key(KeyEvent { code, kind, .. }) = event {
                if kind != KeyEventKind::Release {
                    match code {
                        KeyCode::Char('j') | KeyCode::Down => {
                            let res = navigate_selection(
                                NavDirection::Next,
                                selected.get(),
                                offset.get(),
                                num_items,
                                max_num_list_items,
                            );
                            offset.set(res.offset);
                            selected.set(res.selected);
                        }
                        KeyCode::Char('k') | KeyCode::Up => {
                            let res = navigate_selection(
                                NavDirection::Previous,
                                selected.get(),
                                offset.get(),
                                num_items,
                                max_num_list_items,
                            );
                            offset.set(res.offset);
                            selected.set(res.selected);
                        }
                        KeyCode::Enter => {
                            let selection_value = match &data {
                                SelectableListData::Options(options) => {
                                    if let Some(option) = options.get(selected.get()) {
                                        SelectionValue::OptionId(option.id().clone())
                                    } else {
                                        error!("Invalid option index: {}", selected.get());
                                        return;
                                    }
                                }
                                SelectableListData::StringListItems(_) => {
                                    SelectionValue::Index(selected.get())
                                }
                            };
                            on_selected(Some(selection_value));
                        }
                        KeyCode::Esc => {
                            // Cancel
                            on_selected(None);
                        }
                        _ => {}
                    }
                }
            }
        }
    });

    let current_selection = selected.get();

    let items: Vec<_> = match &props.data {
        SelectableListData::Options(options) => options
            .iter()
            .enumerate()
            .skip(offset.get())
            .map(|(i, option)| option.render(i == current_selection, props.has_focus, props.width))
            .take(max_num_list_items)
            .collect(),
        SelectableListData::StringListItems(items) => items
            .iter()
            .enumerate()
            .skip(offset.get())
            .map(|(i, item)| item.render(i == current_selection, props.has_focus, props.width))
            .take(max_num_list_items)
            .collect(),
    };

    if selected.get() >= num_items {
        if num_items <= max_num_list_items {
            offset.set(0);
        } else {
            offset.set(num_items - max_num_list_items);
        }
        selected.set(num_items.saturating_sub(1));
    }

    let mut debug_item = Vec::new();
    if props.debug_info {
        debug_item.push(
            element! {
                Text(
                    content: format!("Offset: {}, Selected: {}, W: {}, H: {}",
                        offset.get(),
                        selected.get(),
                        props.width.unwrap_or(0),
                        props.height.unwrap_or(0),
                    )
                )
            }
            .into_any(),
        );
    }

    let mut all_items = items;
    all_items.extend(debug_item);

    if let Some(width) = props.width {
        element! {
            View(
                width,
                height: available_height,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Stretch,
            ) {
                #(all_items)
            }
        }
    } else {
        element! {
            View(
                height: available_height,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Stretch,
            ) {
                #(all_items)
            }
        }
    }
}
