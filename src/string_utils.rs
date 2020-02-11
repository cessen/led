//! Misc helpful utility functions for TextBuffer related stuff.

use ropey::{str_utils::byte_to_char_idx, RopeSlice};

pub fn is_line_ending(text: &str) -> bool {
    match text.chars().nth(0) {
        Some(c) if (c >= '\u{000A}' && c <= '\u{000D}') => true,
        Some('\u{0085}') | Some('\u{2028}') | Some('\u{2029}') => true,
        _ => false,
    }
}

pub fn is_whitespace(text: &str) -> bool {
    // TODO: this is a naive categorization of whitespace characters.
    // For better categorization these should be split up into groups
    // based on e.g. breaking vs non-breaking spaces, among other things.
    match text.chars().nth(0) {
        //Some('\u{1680}') | // OGHAM SPACE MARK (here for completeness, but usually displayed as a dash, not as whitespace)
        Some('\u{0009}') | // CHARACTER TABULATION
        Some('\u{0020}') | // SPACE
        Some('\u{00A0}') | // NO-BREAK SPACE
        Some('\u{180E}') | // MONGOLIAN VOWEL SEPARATOR
        Some('\u{202F}') | // NARROW NO-BREAK SPACE
        Some('\u{205F}') | // MEDIUM MATHEMATICAL SPACE
        Some('\u{3000}') | // IDEOGRAPHIC SPACE
        Some('\u{FEFF}') // ZERO WIDTH NO-BREAK SPACE
        => true,

        // EN QUAD, EM QUAD, EN SPACE, EM SPACE, THREE-PER-EM SPACE,
        // FOUR-PER-EM SPACE, SIX-PER-EM SPACE, FIGURE SPACE,
        // PUNCTUATION SPACE, THIN SPACE, HAIR SPACE, ZERO WIDTH SPACE.
        Some(c) if c >= '\u{2000}' && c <= '\u{200B}' => true,

        // None, or not a matching whitespace character.
        _ => false,
    }
}

pub fn char_count(text: &str) -> usize {
    byte_to_char_idx(text, text.len())
}

/// Represents one of the valid Unicode line endings.
/// Also acts as an index into `LINE_ENDINGS`.
#[derive(PartialEq, Copy, Clone)]
pub enum LineEnding {
    None = 0, // No line ending
    CRLF = 1, // CarriageReturn followed by LineFeed
    LF = 2,   // U+000A -- LineFeed
    VT = 3,   // U+000B -- VerticalTab
    FF = 4,   // U+000C -- FormFeed
    CR = 5,   // U+000D -- CarriageReturn
    NEL = 6,  // U+0085 -- NextLine
    LS = 7,   // U+2028 -- Line Separator
    PS = 8,   // U+2029 -- ParagraphSeparator
}

pub fn str_to_line_ending(g: &str) -> LineEnding {
    match g {
        "\u{000D}\u{000A}" => LineEnding::CRLF,
        "\u{000A}" => LineEnding::LF,
        "\u{000B}" => LineEnding::VT,
        "\u{000C}" => LineEnding::FF,
        "\u{000D}" => LineEnding::CR,
        "\u{0085}" => LineEnding::NEL,
        "\u{2028}" => LineEnding::LS,
        "\u{2029}" => LineEnding::PS,

        // Not a line ending
        _ => LineEnding::None,
    }
}

pub fn rope_slice_to_line_ending(g: &RopeSlice) -> LineEnding {
    if let Some(text) = g.as_str() {
        str_to_line_ending(text)
    } else if g == "\u{000D}\u{000A}" {
        LineEnding::CRLF
    } else {
        // Not a line ending
        LineEnding::None
    }
}

pub fn line_ending_to_str(ending: LineEnding) -> &'static str {
    LINE_ENDINGS[ending as usize]
}

/// An array of string literals corresponding to the possible
/// unicode line endings.
pub const LINE_ENDINGS: [&'static str; 9] = [
    "",
    "\u{000D}\u{000A}",
    "\u{000A}",
    "\u{000B}",
    "\u{000C}",
    "\u{000D}",
    "\u{0085}",
    "\u{2028}",
    "\u{2029}",
];

//--------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn char_count_1() {
        let text_1 = "Hello world!";
        let text_2 = "今日はみんなさん！";

        assert_eq!(12, char_count(text_1));
        assert_eq!(9, char_count(text_2));
    }
}
