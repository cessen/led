#![allow(dead_code)]
//! Misc helpful utility functions for TextBuffer related stuff.

use ropey::RopeSlice;
use std::iter::repeat;
use unicode_segmentation::UnicodeSegmentation;

pub fn is_line_ending(text: &str) -> bool {
    match text {
        "\u{000D}\u{000A}" | "\u{000A}" | "\u{000B}" | "\u{000C}" | "\u{000D}" | "\u{0085}"
        | "\u{2028}" | "\u{2029}" => true,

        _ => false,
    }
}

pub fn rope_slice_is_line_ending(text: &RopeSlice) -> bool {
    rope_slice_to_line_ending(text) != LineEnding::None
}

pub fn is_whitespace(text: &str) -> bool {
    // TODO: this is a naive categorization of whitespace characters.
    // For better categorization these should be split up into groups
    // based on e.g. breaking vs non-breaking spaces, among other things.
    match text {
        "\u{0020}" // SPACE
        | "\u{0009}" // CHARACTER TABULATION
        | "\u{00A0}" // NO-BREAK SPACE
        //| "\u{1680}" // OGHAM SPACE MARK (here for completeness, but usually displayed as a dash, not as whitespace)
        | "\u{180E}" // MONGOLIAN VOWEL SEPARATOR
        | "\u{2000}" // EN QUAD
        | "\u{2001}" // EM QUAD
        | "\u{2002}" // EN SPACE
        | "\u{2003}" // EM SPACE
        | "\u{2004}" // THREE-PER-EM SPACE
        | "\u{2005}" // FOUR-PER-EM SPACE
        | "\u{2006}" // SIX-PER-EM SPACE
        | "\u{2007}" // FIGURE SPACE
        | "\u{2008}" // PUNCTUATION SPACE
        | "\u{2009}" // THIN SPACE
        | "\u{200A}" // HAIR SPACE
        | "\u{200B}" // ZERO WIDTH SPACE
        | "\u{202F}" // NARROW NO-BREAK SPACE
        | "\u{205F}" // MEDIUM MATHEMATICAL SPACE
        | "\u{3000}" // IDEOGRAPHIC SPACE
        | "\u{FEFF}" // ZERO WIDTH NO-BREAK SPACE
        => true,
        _ => false,
    }
}

pub fn rope_slice_is_whitespace(text: &RopeSlice) -> bool {
    // TODO: this is a naive categorization of whitespace characters.
    // For better categorization these should be split up into groups
    // based on e.g. breaking vs non-breaking spaces, among other things.

    if let Some(text) = text.as_str() {
        is_whitespace(text)
    } else {
        text == "\u{0020}" // SPACE
        || text == "\u{0009}" // CHARACTER TABULATION
        || text == "\u{00A0}" // NO-BREAK SPACE
        //|| "\u{1680}" // OGHAM SPACE MARK (here for completeness, but usually displayed as a dash, not as whitespace)
        || text == "\u{180E}" // MONGOLIAN VOWEL SEPARATOR
        || text == "\u{2000}" // EN QUAD
        || text == "\u{2001}" // EM QUAD
        || text == "\u{2002}" // EN SPACE
        || text == "\u{2003}" // EM SPACE
        || text == "\u{2004}" // THREE-PER-EM SPACE
        || text == "\u{2005}" // FOUR-PER-EM SPACE
        || text == "\u{2006}" // SIX-PER-EM SPACE
        || text == "\u{2007}" // FIGURE SPACE
        || text == "\u{2008}" // PUNCTUATION SPACE
        || text == "\u{2009}" // THIN SPACE
        || text == "\u{200A}" // HAIR SPACE
        || text == "\u{200B}" // ZERO WIDTH SPACE
        || text == "\u{202F}" // NARROW NO-BREAK SPACE
        || text == "\u{205F}" // MEDIUM MATHEMATICAL SPACE
        || text == "\u{3000}" // IDEOGRAPHIC SPACE
        || text == "\u{FEFF}" // ZERO WIDTH NO-BREAK SPACE
    }
}

pub fn line_ending_count(text: &str) -> usize {
    let mut count = 0;
    for g in UnicodeSegmentation::graphemes(text, true) {
        if is_line_ending(g) {
            count += 1;
        }
    }
    return count;
}

pub fn char_count(text: &str) -> usize {
    let mut count = 0;
    for _ in text.chars() {
        count += 1;
    }
    return count;
}

pub fn grapheme_count(text: &str) -> usize {
    let mut count = 0;
    for _ in UnicodeSegmentation::graphemes(text, true) {
        count += 1;
    }
    return count;
}

pub fn grapheme_count_is_less_than(text: &str, n: usize) -> bool {
    let mut count = 0;
    for _ in UnicodeSegmentation::graphemes(text, true) {
        count += 1;
        if count >= n {
            return false;
        }
    }

    return true;
}

pub fn grapheme_and_line_ending_count(text: &str) -> (usize, usize) {
    let mut grapheme_count = 0;
    let mut line_ending_count = 0;

    for g in UnicodeSegmentation::graphemes(text, true) {
        grapheme_count += 1;
        if is_line_ending(g) {
            line_ending_count += 1;
        }
    }

    return (grapheme_count, line_ending_count);
}

pub fn char_pos_to_byte_pos(text: &str, pos: usize) -> usize {
    let mut i: usize = 0;

    for (offset, _) in text.char_indices() {
        if i == pos {
            return offset;
        }
        i += 1;
    }

    if i == pos {
        return text.len();
    }

    panic!("char_pos_to_byte_pos(): char position off the end of the string.");
}

pub fn grapheme_pos_to_byte_pos(text: &str, pos: usize) -> usize {
    let mut i: usize = 0;

    for (offset, _) in UnicodeSegmentation::grapheme_indices(text, true) {
        if i == pos {
            return offset;
        }
        i += 1;
    }

    if i == pos {
        return text.len();
    }

    panic!("grapheme_pos_to_byte_pos(): grapheme position off the end of the string.");
}

/// Inserts the given text into the given string at the given grapheme index.
pub fn insert_text_at_grapheme_index(s: &mut String, text: &str, pos: usize) {
    // Find insertion position in bytes
    let byte_pos = grapheme_pos_to_byte_pos(&s[..], pos);

    // Get byte vec of string
    let byte_vec = unsafe { s.as_mut_vec() };

    // Grow data size
    byte_vec.extend(repeat(0).take(text.len()));

    // Move old bytes forward
    // TODO: use copy_memory()...?
    let mut from = byte_vec.len() - text.len();
    let mut to = byte_vec.len();
    while from > byte_pos {
        from -= 1;
        to -= 1;

        byte_vec[to] = byte_vec[from];
    }

    // Copy new bytes in
    // TODO: use copy_memory()
    let mut i = byte_pos;
    for g in UnicodeSegmentation::graphemes(text, true) {
        for b in g.bytes() {
            byte_vec[i] = b;
            i += 1
        }
    }
}

/// Removes the text between the given grapheme indices in the given string.
pub fn remove_text_between_grapheme_indices(s: &mut String, pos_a: usize, pos_b: usize) {
    // Bounds checks
    assert!(
        pos_a <= pos_b,
        "remove_text_between_grapheme_indices(): pos_a must be less than or equal to pos_b."
    );

    if pos_a == pos_b {
        return;
    }

    // Find removal positions in bytes
    // TODO: get both of these in a single pass
    let byte_pos_a = grapheme_pos_to_byte_pos(&s[..], pos_a);
    let byte_pos_b = grapheme_pos_to_byte_pos(&s[..], pos_b);

    // Get byte vec of string
    let byte_vec = unsafe { s.as_mut_vec() };

    // Move bytes to fill in the gap left by the removed bytes
    let mut from = byte_pos_b;
    let mut to = byte_pos_a;
    while from < byte_vec.len() {
        byte_vec[to] = byte_vec[from];

        from += 1;
        to += 1;
    }

    // Remove data from the end
    let final_text_size = byte_vec.len() + byte_pos_a - byte_pos_b;
    byte_vec.truncate(final_text_size);
}

/// Splits a string into two strings at the grapheme index given.
/// The first section of the split is stored in the original string,
/// while the second section of the split is returned as a new string.
pub fn split_string_at_grapheme_index(s1: &mut String, pos: usize) -> String {
    let mut s2 = String::new();

    // Code block to contain the borrow of s2
    {
        let byte_pos = grapheme_pos_to_byte_pos(&s1[..], pos);

        let byte_vec_1 = unsafe { s1.as_mut_vec() };
        let byte_vec_2 = unsafe { s2.as_mut_vec() };

        byte_vec_2.extend((&byte_vec_1[byte_pos..]).iter().cloned());
        byte_vec_1.truncate(byte_pos);
    }

    return s2;
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
        // ==============
        // Line endings
        // ==============
        //
        // CRLF
        "\u{000D}\u{000A}" => {
            return LineEnding::CRLF;
        }

        // LF
        "\u{000A}" => {
            return LineEnding::LF;
        }

        // VT
        "\u{000B}" => {
            return LineEnding::VT;
        }

        // FF
        "\u{000C}" => {
            return LineEnding::FF;
        }

        // CR
        "\u{000D}" => {
            return LineEnding::CR;
        }

        // NEL
        "\u{0085}" => {
            return LineEnding::NEL;
        }

        // LS
        "\u{2028}" => {
            return LineEnding::LS;
        }

        // PS
        "\u{2029}" => {
            return LineEnding::PS;
        }

        // Not a line ending
        _ => {
            return LineEnding::None;
        }
    }
}

pub fn rope_slice_to_line_ending(g: &RopeSlice) -> LineEnding {
    if let Some(text) = g.as_str() {
        str_to_line_ending(text)
    } else if g == "\u{000D}\u{000A}" {
        LineEnding::CRLF
    } else if g == "\u{000A}" {
        LineEnding::LF
    } else if g == "\u{000B}" {
        LineEnding::VT
    } else if g == "\u{000C}" {
        LineEnding::FF
    } else if g == "\u{000D}" {
        LineEnding::CR
    } else if g == "\u{0085}" {
        LineEnding::NEL
    } else if g == "\u{2028}" {
        LineEnding::LS
    } else if g == "\u{2029}" {
        LineEnding::PS
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
