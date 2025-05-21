/// Represents all possible HTML input types
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum InputType {
    Button,
    Checkbox,
    Color,
    Date,
    DatetimeLocal,
    Email,
    File,
    Hidden,
    Image,
    Month,
    Number,
    Password,
    Radio,
    Range,
    Reset,
    Search,
    Submit,
    Tel,
    Text,
    Time,
    Url,
    Week,
}

impl InputType {
    /// Creates an InputType from a string slice
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "button" => Some(InputType::Button),
            "checkbox" => Some(InputType::Checkbox),
            "color" => Some(InputType::Color),
            "date" => Some(InputType::Date),
            "datetime-local" => Some(InputType::DatetimeLocal),
            "email" => Some(InputType::Email),
            "file" => Some(InputType::File),
            "hidden" => Some(InputType::Hidden),
            "image" => Some(InputType::Image),
            "month" => Some(InputType::Month),
            "number" => Some(InputType::Number),
            "password" => Some(InputType::Password),
            "radio" => Some(InputType::Radio),
            "range" => Some(InputType::Range),
            "reset" => Some(InputType::Reset),
            "search" => Some(InputType::Search),
            "submit" => Some(InputType::Submit),
            "tel" => Some(InputType::Tel),
            "text" => Some(InputType::Text),
            "time" => Some(InputType::Time),
            "url" => Some(InputType::Url),
            "week" => Some(InputType::Week),
            _ => None,
        }
    }

    /// Converts InputType to lowercase string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            InputType::Button => "button",
            InputType::Checkbox => "checkbox",
            InputType::Color => "color",
            InputType::Date => "date",
            InputType::DatetimeLocal => "datetime-local",
            InputType::Email => "email",
            InputType::File => "file",
            InputType::Hidden => "hidden",
            InputType::Image => "image",
            InputType::Month => "month",
            InputType::Number => "number",
            InputType::Password => "password",
            InputType::Radio => "radio",
            InputType::Range => "range",
            InputType::Reset => "reset",
            InputType::Search => "search",
            InputType::Submit => "submit",
            InputType::Tel => "tel",
            InputType::Text => "text",
            InputType::Time => "time",
            InputType::Url => "url",
            InputType::Week => "week",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        assert_eq!(InputType::from_str("button"), Some(InputType::Button));
        assert_eq!(InputType::from_str("BUTTON"), Some(InputType::Button));
        assert_eq!(InputType::from_str("invalid"), None);
    }

    #[test]
    fn test_as_str() {
        assert_eq!(InputType::Button.as_str(), "button");
        assert_eq!(InputType::DatetimeLocal.as_str(), "datetime-local");
    }
}
