use iocraft::prelude::*;

#[component]
pub fn Logo() -> impl Into<AnyElement<'static>> {
    // # https://patorjk.com/software/taag/#p=display&f=Epic&t=NixBlitz
    // # https://patorjk.com/software/taag/#p=display&f=Ivrit&t=NixBlitz <-- current
    element! {
        View(
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Stretch
        ) {
            // The unicode is needed to prevent the text from centering.
            // I suspect that the Text component removes whitespace from the end of the string.
            Text(content: "   _   _        ____  _   _        \u{200b}".to_string())
            Text(content: r" | \ | (_)_  _| __ )| (_) |_ ____".to_string())
            Text(content: r" |  \| | \ \/ /  _ \| | | __|_  /".to_string())
            Text(content: r"| |\  | |>  <| |_) | | | |_ / / ".to_string())
            Text(content: r" |_| \_|_/_/\_\____/|_|_|\__/___|".to_string())
        }
    }
}
