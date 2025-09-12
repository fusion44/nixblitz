use crate::errors::StringErrors;

pub trait GetStringOrCliError {
    fn get_or_err(&self) -> Result<&str, StringErrors>;
}

/// Truncates text to fit within a specified width, optionally adding a prefix.
///
/// This function handles text truncation safely, especially with multi-byte Unicode characters.
///
/// # Parameters
///
/// * `text`: The input text to be truncated.
/// * `prefix`: Optional prefix to add to the text.
/// * `width`: Optional maximum width in characters.
///
/// # Returns
///
/// A `String` containing the formatted and possibly truncated text.
pub fn truncate_text(text: &str, prefix: Option<&str>, width: Option<usize>) -> String {
    let prefix = prefix.unwrap_or("");
    let full_prefix = if prefix.is_empty() {
        String::new()
    } else {
        format!("{} ", prefix)
    };

    let Some(max_width) = width else {
        return format!("{}{}", full_prefix, text);
    };

    if max_width == 0 {
        return String::new();
    }

    let combined_text = format!("{}{}", full_prefix, text);
    if combined_text.chars().count() <= max_width {
        return combined_text;
    }

    let ellipsis = "…";
    let truncated: String = combined_text
        .chars()
        .take(max_width.saturating_sub(ellipsis.chars().count()))
        .collect();

    format!("{}{}", truncated, ellipsis)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_truncation_when_no_width_is_given() {
        let text = "This is a long text";
        let prefix = "Prefix";
        assert_eq!(
            truncate_text(text, Some(prefix), None),
            "Prefix This is a long text"
        );
    }

    #[test]
    fn test_no_truncation_when_text_fits_within_width() {
        let text = "Short text";
        let prefix = "Info";
        assert_eq!(
            truncate_text(text, Some(prefix), Some(30)),
            "Info Short text"
        );
    }

    #[test]
    fn test_basic_truncation_with_prefix() {
        let text = "This is a very long text that needs to be cut short";
        let prefix = "Attention";
        // Final length must be exactly 30
        assert_eq!(
            truncate_text(text, Some(prefix), Some(30)),
            "Attention This is a very long…"
        );
    }

    #[test]
    fn test_truncation_without_prefix() {
        let text = "This is a very long text that needs to be cut short";
        // Final length must be exactly 20
        assert_eq!(truncate_text(text, None, Some(20)), "This is a very long…");
    }

    #[test]
    fn test_unicode_safety_truncation() {
        let text = "Here are some emojis 👍 and some text 🎉";
        let prefix = "Unicode";

        // The function correctly calculates the final string:
        // 1. Combined text starts with "Unicode Here are some emojis 👍..."
        // 2. It must be truncated to a max width of 30.
        // 3. To fit the "…" (1 char), the content part is truncated to 29 chars.
        // 4. The first 29 chars of the combined text are "Unicode Here are some emojis ".
        // 5. The final result is "Unicode Here are some emojis " + "…".
        let expected = "Unicode Here are some emojis …";

        // The total character count of `expected` is now 30, matching the width limit.
        assert_eq!(truncate_text(text, Some(prefix), Some(30)), expected);
    }

    #[test]
    fn test_width_smaller_than_prefix() {
        let text = "This text will not be visible";
        let prefix = "VeryLongPrefix";
        // The prefix itself ("VeryLongPrefix ") is 15 chars.
        // Truncating it to 14 chars + ellipsis makes 15.
        assert_eq!(
            truncate_text(text, Some(prefix), Some(15)),
            "VeryLongPrefix…"
        );
    }

    #[test]
    fn test_empty_text_input() {
        let text = "";
        let prefix = "Info";
        assert_eq!(truncate_text(text, Some(prefix), Some(20)), "Info ");
    }

    #[test]
    fn test_empty_prefix() {
        let text = "Text without a prefix label";
        assert_eq!(truncate_text(text, Some(""), Some(12)), "Text withou…");
    }

    #[test]
    fn test_truncation_to_zero_width() {
        let text = "Some text";
        assert_eq!(truncate_text(text, None, Some(0)), "");
    }

    #[test]
    fn test_truncation_to_one_width() {
        let text = "Some text";
        assert_eq!(truncate_text(text, None, Some(1)), "…");
    }
}
