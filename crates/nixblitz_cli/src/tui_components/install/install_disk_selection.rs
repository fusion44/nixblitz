use iocraft::prelude::*;
use nixblitz_core::DiskInfo;

const BYTES_IN_GB: u64 = 1024 * 1024 * 1024;

#[derive(Default, Props)]
pub struct InstallDiskSelectionProps {
    pub disks: Vec<DiskInfo>,
    pub on_select: Option<Handler<'static, String>>,
}

#[component]
pub fn InstallDiskSelection(
    props: &mut InstallDiskSelectionProps,
    hooks: &mut Hooks,
) -> impl Into<AnyElement<'static>> {
    let mut selected_index = hooks.use_state(|| 0);

    hooks.use_terminal_events({
        let num_disks = props.disks.len();
        let disks = props.disks.clone();
        let mut on_select = props.on_select.take();

        move |event| {
            if let TerminalEvent::Key(KeyEvent {
                code,
                kind: KeyEventKind::Press,
                ..
            }) = event
            {
                match code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        selected_index.set(if *selected_index.read() > 0 {
                            *selected_index.read() - 1
                        } else {
                            num_disks - 1
                        });
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        let new_value = (*selected_index.read() + 1) % num_disks;
                        selected_index.set(new_value);
                    }
                    KeyCode::Enter => {
                        if let Some(cb) = &mut on_select {
                            if let Some(disk) = disks.get(*selected_index.read()) {
                                cb(disk.path.clone());
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    });

    let disk_elements = props
        .disks
        .iter()
        .enumerate()
        .map(|(i, disk)| {
            let is_selected = *selected_index.read() == i;
            let display_text = format!(
                "{} {} ({} GB)",
                if is_selected { ">" } else { " " },
                disk.path,
                disk.size_bytes / BYTES_IN_GB
            );
            let background_color = if is_selected {
                Color::Cyan
            } else {
                Color::Reset
            };
            element! {
                View(
                    background_color,
                ) {
                    Text(
                        content: display_text,
                        color: if is_selected { Color::Black } else { Color::White }
                    )
                }
            }
        })
        .collect::<Vec<_>>();

    element! {
        View(flex_direction: FlexDirection::Column, align_items: AlignItems::Center) {
            Text(content: "Select Installation Disk".to_string(), color: Color::Cyan)
            Text(content: "Use arrow keys to select and Enter to confirm.".to_string())
            #(disk_elements)
        }
    }
}
