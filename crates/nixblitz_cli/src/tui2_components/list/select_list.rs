use iocraft::prelude::*;
use log::error;
use nixblitz_core::string_list_data::StringListOptionItem;

use crate::tui2_components::{ListItem, NavDirection, navigate_selection};

#[derive(Default, Props)]
pub struct SelectListProps {
    pub has_focus: bool,
    pub on_selected: Handler<'static, usize>,
    pub options: Vec<StringListOptionItem>,
}

#[component]
pub fn SelectList(props: &mut SelectListProps, mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    if props.options.is_empty() {
        error!("No options given to StringListPopup");
        return element! {
            View {
                Text(content: "ERROR: No options available")
            }
        };
    }

    let (_, height) = hooks.use_terminal_size();
    let height = height.min(20);
    let mut selected = hooks.use_state(|| 0);
    let num_opts = props.options.len();
    let max_num_list_items = height as usize;
    let mut offset = hooks.use_state(|| 0);

    if props.has_focus {
        hooks.use_terminal_events({
            let mut on_selected = props.on_selected.take();

            move |event| {
                if let TerminalEvent::Key(KeyEvent { code, kind, .. }) = event {
                    if kind != KeyEventKind::Release {
                        match code {
                            KeyCode::Char('j') | KeyCode::Down => {
                                let res = navigate_selection(
                                    NavDirection::Next,
                                    selected.get(),
                                    offset.get(),
                                    num_opts,
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
                                    num_opts,
                                    max_num_list_items,
                                );
                                offset.set(res.offset);
                                selected.set(res.selected);
                            }
                            KeyCode::Enter => {
                                on_selected(selected.get());
                            }
                            _ => {}
                        }
                    }
                }
            }
        });
    }

    let current_selection = selected.get();
    let items: Vec<_> = props
        .options
        .iter()
        .enumerate()
        .skip(offset.get())
        .map(|(i, o)| {
            element! {
                ListItem(
                    item: o.clone(),
                    is_selected: i == current_selection,
                )
            }
        })
        .take(max_num_list_items)
        .collect();

    element! {
        View(
            height,
            flex_direction: FlexDirection::Column,
            border_style: BorderStyle::None,
        ) {
            #(items)
        }
    }
}
