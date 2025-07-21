use iocraft::prelude::*;

#[derive(Default, Props)]
pub struct CustomTextInputProps {
    pub multiline: bool,
    pub has_focus: bool,
    pub value: String,
    pub on_change: Handler<'static, String>,
    pub masked: bool,
}

#[component]
pub fn CustomTextInput(props: &mut CustomTextInputProps) -> impl Into<AnyElement<'static>> {
    let display_value = if props.masked {
        "*".repeat(props.value.len())
    } else {
        props.value.clone()
    };

    let on_change = props.on_change.take();
    element! {
        TextInput(
            multiline: props.multiline,
            has_focus: props.has_focus,
            value: display_value,
            on_change,
        )
    }
}
