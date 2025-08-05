use std::time::Duration;

use iocraft::prelude::*;
use tokio::time;

/// Defines the visual style of the spinner animation.
#[derive(Default, Copy, Clone, Debug, PartialEq, Eq)]
pub enum SpinnerStyle {
    #[default]
    BrailleCircle,
    BrailleDots,
    Lines,
    Moon,
    Bar,
    Arrows,
    Corners,
}

/// Defines the animation speed for the spinner.
#[derive(Default, Copy, Clone, Debug, PartialEq, Eq)]
pub enum SpinnerSpeed {
    Slow,
    #[default]
    Normal,
    Fast,
}

/// Properties for the `Spinner` component.
#[derive(Props)]
pub struct SpinnerProps {
    /// The visual style of the spinner animation.
    pub style: SpinnerStyle,
    /// The color of the spinner text.
    pub color: Color,
    /// The animation speed of the spinner.
    pub speed: SpinnerSpeed,
}

impl Default for SpinnerProps {
    fn default() -> Self {
        Self {
            style: Default::default(),
            color: Color::White,
            speed: Default::default(),
        }
    }
}

/// A component that renders a customizable, single-character CLI spinner.
#[component]
pub fn Spinner(props: &SpinnerProps, hooks: &mut Hooks) -> impl Into<AnyElement<'static>> {
    let chars: &[&'static str] = match props.style {
        SpinnerStyle::BrailleCircle => &["⠟", "⠯", "⠷", "⠾", "⠽", "⠻"],
        SpinnerStyle::BrailleDots => &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
        SpinnerStyle::Lines => &["|", "/", "—", "\\"],
        SpinnerStyle::Moon => &["◐", "◓", "◑", "◒"],
        SpinnerStyle::Bar => &[" ", "▂", "▃", "▄", "▅", "▆", "▅", "▄", "▂", " "],
        SpinnerStyle::Arrows => &["↑", "↗", "→", "↘", "↓", "↙", "←", "↖"],
        SpinnerStyle::Corners => &["┌", "┐", "┘", "└"],
    };

    let interval_duration = match props.speed {
        SpinnerSpeed::Slow => Duration::from_millis(200),
        SpinnerSpeed::Normal => Duration::from_millis(100),
        SpinnerSpeed::Fast => Duration::from_millis(50),
    };

    let mut current = hooks.use_state(|| 0);

    hooks.use_future(async move {
        let mut interval = time::interval(interval_duration);
        loop {
            interval.tick().await;
            current.set((current.get() + 1) % chars.len());
        }
    });

    let char_to_render = chars[current.get()];

    element! {
        View(width: 1, height: 1) {
            Text(content: char_to_render, color: props.color)
        }
    }
}
