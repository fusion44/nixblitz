use std::sync::Arc;

use iocraft::prelude::*;
use log::error;
use nixblitz_core::{
    OPTION_TITLES,
    option_data::{GetOptionId, OptionData, OptionId},
    string_list_data::StringListOptionItem,
};

use crate::{
    errors::CliError,
    tui2_components::{
        ListItem, NavDirection, get_selected_char, navigate_selection,
        utils::{
            DEFAULT_MAX_HEIGHT, RenderWithWidth, SelectableItem, format_bool_subtitle,
            get_focus_border_color, get_selected_item_color,
        },
    },
};

impl SelectableItem for StringListOptionItem {
    type SelectionValue = usize;

    fn render(&self, is_selected: bool, component_focused: bool) -> AnyElement<'static> {
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

    fn render(&self, is_selected: bool, component_focused: bool) -> AnyElement<'static> {
        self.render_with_width(is_selected, component_focused, None)
    }
}

impl RenderWithWidth for OptionData {
    fn render_with_width(
        &self,
        is_selected: bool,
        component_focused: bool,
        max_width: Option<u16>,
    ) -> AnyElement<'static> {
        let char = get_selected_char(is_selected);
        let background_color = get_selected_item_color(is_selected, component_focused);
        let id = self.id();
        let title = OPTION_TITLES
            .get(id)
            .ok_or(CliError::OptionTitleRetrievalError(id.to_string()))
            .unwrap();

        let truncate_text = |text: &str, prefix: &str| {
            if let Some(width) = max_width {
                let available_width = width.saturating_sub(4) as usize; // Account for border + padding
                let full_text = format!("{} {}", prefix, text);
                if full_text.len() > available_width {
                    format!(
                        "{} {}...",
                        prefix,
                        &text[..available_width.saturating_sub(prefix.len() + 4)]
                    )
                } else {
                    full_text
                }
            } else {
                format!("{} {}", prefix, text)
            }
        };

        match self {
            OptionData::Bool(b) => {
                let subtitle = format_bool_subtitle(b.value());
                let title_text = truncate_text(title, &char.to_string());
                let subtitle_text = truncate_text(&subtitle, &char.to_string());

                element! {
                    View(
                        flex_direction: FlexDirection::Column,
                        background_color,
                    ) {
                        Text(content: title_text)
                        Text(content: subtitle_text)
                    }
                }
                .into_any()
            }
            _ => {
                let title_text = truncate_text(title, &char.to_string());
                let subtitle_text = truncate_text("subtitle", &char.to_string());

                element! {
                    View(
                        flex_direction: FlexDirection::Column,
                        background_color,
                    ) {
                        Text(content: title_text)
                        Text(content: subtitle_text)
                    }
                }
                .into_any()
            }
        }
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
    pub on_selected: Handler<'static, SelectionValue>,
    pub data: SelectableListData,
    pub show_border: bool,
    pub max_height: Option<u16>,
    pub width: Option<u16>,
    pub debug_info: bool,
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

    let (_, height) = hooks.use_terminal_size();
    let max_height = props.max_height.unwrap_or(DEFAULT_MAX_HEIGHT);
    let height = height.min(max_height);

    let num_items = props.data.len();
    let max_num_list_items = match &props.data {
        SelectableListData::Options(_) => (height as usize / 2) - 1, // minus one for the borders
        SelectableListData::StringListItems(_) => height as usize,
    };

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
                            on_selected(selection_value);
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
            .map(|(i, option)| {
                option.render_with_width(i == current_selection, props.has_focus, props.width)
            })
            .take(max_num_list_items)
            .collect(),
        SelectableListData::StringListItems(items) => items
            .iter()
            .enumerate()
            .skip(offset.get())
            .map(|(i, item)| item.render(i == current_selection, props.has_focus))
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
                Text(content: format!("Offset: {}, Selected: {}", offset.get(), selected.get()))
            }
            .into_any(),
        );
    }

    let mut all_items = items;
    all_items.extend(debug_item);

    if props.show_border {
        let border_color = get_focus_border_color(props.has_focus);
        if let Some(width) = props.width {
            element! {
                View(
                    width,
                    height: height + 2,
                    flex_direction: FlexDirection::Column,
                    border_style: BorderStyle::Round,
                    border_color,
                ) {
                    View(flex_direction: FlexDirection::Column) {
                        #(all_items)
                    }
                }
            }
        } else {
            element! {
                View(
                    height: height + 2,
                    flex_direction: FlexDirection::Column,
                    border_style: BorderStyle::Round,
                    border_color,
                ) {
                    View(flex_direction: FlexDirection::Column) {
                        #(all_items)
                    }
                }
            }
        }
    } else if let Some(width) = props.width {
        element! {
            View(
                width,
                height,
                flex_direction: FlexDirection::Column,
                border_style: BorderStyle::None,
            ) {
                View(flex_direction: FlexDirection::Column) {
                    #(all_items)
                }
            }
        }
    } else {
        element! {
            View(
                height,
                flex_direction: FlexDirection::Column,
                border_style: BorderStyle::None,
            ) {
                View(flex_direction: FlexDirection::Column) {
                    #(all_items)
                }
            }
        }
    }
}
