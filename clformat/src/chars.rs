//! Helpers for the `~C` directive - printing characters by name.

/// Returns the "long" name of a character, as used by `~:C` and `~@C`, for
/// characters that commonly have a name (whitespace and other control
/// characters). Returns `None` for characters that are just printed as
/// themselves.
pub fn char_name(c: char) -> Option<&'static str> {
    match c {
        ' ' => Some("Space"),
        '\n' => Some("Newline"),
        '\t' => Some("Tab"),
        '\r' => Some("Return"),
        '\0' => Some("Null"),
        '\u{7}' => Some("Bell"),
        '\u{8}' => Some("Backspace"),
        '\u{c}' => Some("Page"),
        '\u{1b}' => Some("Escape"),
        '\u{7f}' => Some("Rubout"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_known_characters() {
        assert_eq!(Some("Space"), char_name(' '));
        assert_eq!(Some("Newline"), char_name('\n'));
        assert_eq!(Some("Tab"), char_name('\t'));
    }

    #[test]
    fn unknown_characters_have_no_name() {
        assert_eq!(None, char_name('a'));
        assert_eq!(None, char_name('9'));
    }
}
