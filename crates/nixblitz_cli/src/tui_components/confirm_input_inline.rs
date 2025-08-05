use iocraft::prelude::*;

use crate::tui_components::Confirm;

/// Props for the `ConfirmInputInline` component.
#[derive(Default, Props)]
pub struct ConfirmInputInlineProps<'a> {
    /// The question or prompt displayed above the confirmation options.
    pub title: String,

    /// A mutable reference to a boolean that will be updated with the user's final choice.
    /// (`true` for "Yes", `false` for "No"). Wrapped in an `Option` to implement `Default`.
    pub value_out: Option<&'a mut bool>,
}

/// A self-contained component for handling a yes/no confirmation prompt.
///
/// This component is designed for **inline usage** within a CLI command. It takes full
/// control of the terminal, listens for key presses to manage its own state, and exits
/// the application when the user makes a selection. The result is passed back to the
/// caller via a mutable reference in the `value_out` prop.
///
/// ## Key Bindings ⌨️
/// - `y`: Selects "Yes".
/// - `n`: Selects "No".
/// - `h`, `l`, `←`, `→`: Toggles the current selection.
/// - `Enter`: Confirms the current selection and exits.
/// - `q`, `Esc`: Selects "No", confirms, and exits.
///
/// ## Example
/// ```rust
/// // This example assumes it's run within an iocraft application context.
/// let mut user_confirmed = false;
///
/// // This will render the prompt, wait for user input, and then exit.
/// let mut output = false;
/// if !decision {
///     let _ = element! {
///         ConfirmInputInline(
///             title: "Do you want to proceed?".to_string(),
///             value_out: &mut output
///         )
///     }
///     .render_loop()
///     .await;
/// }
///
/// // After the app exits, `user_confirmed` will hold the user's choice.
/// println!("User selected: {}", user_confirmed);
/// ```
#[component]
pub fn ConfirmInputInline<'a>(
    props: &mut ConfirmInputInlineProps<'a>,
    hooks: &mut Hooks,
) -> impl Into<AnyElement<'static>> {
    let mut system = hooks.use_context_mut::<SystemContext>();
    let mut selection = hooks.use_state(|| false);
    let mut should_submit = hooks.use_state(|| false);

    hooks.use_terminal_events({
        move |event| match event {
            TerminalEvent::Key(KeyEvent { code, kind, .. }) if kind != KeyEventKind::Release => {
                match code {
                    KeyCode::Char('y') => {
                        selection.set(true);
                    }
                    KeyCode::Char('n') => {
                        selection.set(false);
                    }
                    KeyCode::Left | KeyCode::Char('h') | KeyCode::Right | KeyCode::Char('l') => {
                        selection.set(!selection.get());
                    }
                    KeyCode::Enter => {
                        should_submit.set(true);
                    }
                    KeyCode::Esc | KeyCode::Char('q') => {
                        selection.set(false);
                        should_submit.set(true);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    });

    if should_submit.get() {
        if let Some(value_out) = props.value_out.as_mut() {
            **value_out = selection.get();
        }
        system.exit();
        element!(View)
    } else {
        element! {
            View(
                flex_direction: FlexDirection::Column,
                height: 2,
            ){
                Text(
                    content: props.title.clone(),
                    align: TextAlign::Center,
                )
                Confirm(value: selection.get())
            }
        }
    }
}
