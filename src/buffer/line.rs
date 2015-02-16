#![allow(dead_code)]

use std::mem;
use super::rope::{Rope, RopeGraphemeIter};
use string_utils::{is_line_ending, grapheme_count};


/// A single line of text
pub struct Line {
    text: Rope, // The text data, stored as UTF8
    pub ending: LineEnding, // The type of line ending, if any
}


impl Line {
    /// Creates a new empty Line
    pub fn new() -> Line {
        Line {
            text: Rope::new(),
            ending: LineEnding::None,
        }
    }
    
    
    /// Creates a new Line from a str.
    pub fn new_from_str(text: &str) -> Line {
        let mut ending = LineEnding::None;
        let mut end_pos = 0;
        
        // Find the slice before the line ending, if any
        for g in text.graphemes(true) {
            match g {
                //==============
                // Line endings
                //==============
                
                // CRLF
                "\u{000D}\u{000A}" => {
                    ending = LineEnding::CRLF;
                    break;
                },
                
                // LF
                "\u{000A}" => {
                    ending = LineEnding::LF;
                    break;
                },
                
                // VT
                "\u{000B}" => {
                    ending = LineEnding::VT;
                    break;
                },
                
                // FF
                "\u{000C}" => {
                    ending = LineEnding::FF;
                    break;
                },
                
                // CR
                "\u{000D}" => {
                    ending = LineEnding::CR;
                    break;
                },
                
                // NEL
                "\u{0085}" => {
                    ending = LineEnding::NEL;
                    break;
                },
                
                // LS
                "\u{2028}" => {
                    ending = LineEnding::LS;
                    break;
                },
                
                // PS
                "\u{2029}" => {
                    ending = LineEnding::PS;
                    break;
                },
                
                //==================
                // Other characters
                //==================
                
                _ => {
                    end_pos += g.len();
                }
            }
        }
        
        // Create and return Line
        return Line {
            text: Rope::new_from_str(&text[..end_pos]),
            ending: ending,
        };
    }
    
    
    pub fn new_from_str_with_count_unchecked(text: &str, count: usize) -> Line {
        let mut ending = LineEnding::None;
        
        let bytes = text.as_bytes();
        
        // Check for line ending
        let mut le_size: usize = 0;
        let text_size = text.len();
        if text.len() >= 3 {
            match &text[(text_size-3)..] {
                // LS
                "\u{2028}" => {
                    ending = LineEnding::LS;
                    le_size = 3;
                },
                
                // PS
                "\u{2029}" => {
                    ending = LineEnding::PS;
                    le_size = 3;
                },
                
                _ => {}
            }
        }
        
        if le_size == 0 && text.len() >= 2 {
            match &text[(text_size-2)..] {
                // CRLF
                "\u{000D}\u{000A}" => {
                    ending = LineEnding::CRLF;
                    le_size = 2;
                },
                
                _ => {}
            }
        }
        
        if le_size == 0 && text.len() >= 1 {
            match &text[(text_size-1)..] {
                // LF
                "\u{000A}" => {
                    ending = LineEnding::LF;
                    le_size = 1;
                },
                
                // VT
                "\u{000B}" => {
                    ending = LineEnding::VT;
                    le_size = 1;
                },
                
                // FF
                "\u{000C}" => {
                    ending = LineEnding::FF;
                    le_size = 1;
                },
                
                // CR
                "\u{000D}" => {
                    ending = LineEnding::CR;
                    le_size = 1;
                },
                
                // NEL
                "\u{0085}" => {
                    ending = LineEnding::NEL;
                    le_size = 1;
                },
                
                _ => {}
            }
        }
        
        // Create and return Line
        let cnt = if ending == LineEnding::None { count } else { count - 1 };
        return Line {
            text: Rope::new_from_str_with_count(&text[..(bytes.len()-le_size)], cnt),
            ending: ending,
        };
    }
    
    
    /// Creates a new Line from a string.
    /// Does not check to see if the string has internal newlines.
    /// This is primarily used for efficient loading of files.
    pub fn new_from_string_unchecked(text: String) -> Line {
        // TODO: this can be smarter, and can pass the string
        // directly to the Rope after taking off any line
        // endings.
        return Line::new_from_str_with_count_unchecked(text.as_slice(), grapheme_count(text.as_slice()));
    }
    
    
    /// Returns the total number of unicode graphemes in the line
    pub fn grapheme_count(&self) -> usize {
        let mut count = self.text.grapheme_count();
        match self.ending {
            LineEnding::None => {},
            _ => {count += 1;}
        }
        return count;
    }
    
    
    /// Returns the total number of unicode graphemes in the line,
    /// not counting the line ending grapheme, if any.
    pub fn grapheme_count_sans_line_ending(&self) -> usize {
        self.text.grapheme_count()
    }
    
    
    pub fn grapheme_at_index<'a>(&'a self, index: usize) -> &'a str {
        // TODO: we don't have to iterate over the entire line
        // anymore because we're using a rope now.  Update.
        let mut i = 0;
        
        for g in self.grapheme_iter() {
            if i == index {
                return g;
            }
            else {
                i += 1;
            }
        }
        
        // Should never get here
        panic!("Line::grapheme_at_index(): index past end of line.");
    }
        
    
    /// Returns a string containing the line's text
    pub fn to_string(&self) -> String {
        let s = self.text.to_string();
        return s;
    }
    
    
    /// Inserts `text` at grapheme index `pos`.
    /// NOTE: panics if it encounters a line ending in the text.
    pub fn insert_text(&mut self, text: &str, pos: usize) {
        // Check for line endings
        for g in text.graphemes(true) {
            if is_line_ending(g) {
                panic!("Line::insert_text(): line ending in inserted text.");
            }
        }
        
        // Insert text
        self.text.insert_text_at_grapheme_index(text, pos);
    }
    
    
    /// Appends `text` to the end of line, just before the line ending (if
    /// any).
    /// NOTE: panics if it encounters a line ending in the text.
    pub fn append_text(&mut self, text: &str) {
        // Check for line endings
        for g in text.graphemes(true) {
            if is_line_ending(g) {
                panic!("Line::append_text(): line ending in inserted text.");
            }
        }
        
        // Append text
        let gc = self.text.grapheme_count();
        self.text.insert_text_at_grapheme_index(text, gc);
    }
    
    
    /// Remove the text between grapheme positions 'pos_a' and 'pos_b'.
    pub fn remove_text(&mut self, pos_a: usize, pos_b: usize) {
        self.text.remove_text_between_grapheme_indices(pos_a, pos_b);
    }
    
    
    /// Insert a line break into the line, splitting it into two.
    /// This line stays as the first part of the split.  The second
    /// part is returned.
    pub fn split(&mut self, ending: LineEnding, pos: usize) -> Line {
        // TODO: change code to use Rope
        let mut other = Line::new();
        
        // Inserting at very beginning: special cased for efficiency
        if pos == 0 {
            mem::swap(self, &mut other);
            self.ending = ending;
        }
        // Otherwise, general case
        else {
            // Split the text
            other.text = self.text.split(pos);
            
            // Set the line endings appropriately
            other.ending = self.ending;
            self.ending = ending;
        }
        
        return other;
    }
    
    
    /// Appends another line to the end of this one, consuming the other
    /// line.
    /// Note that the resulting line ending is the ending of the other
    /// line, if any.
    pub fn append(&mut self, other: Line) {
        self.ending = other.ending;
        self.text.append(other.text);
    }
    
    
    /// Returns an iterator over the graphemes of the line
    pub fn grapheme_iter<'a>(&'a self) -> LineGraphemeIter<'a> {
        LineGraphemeIter {
            graphemes: self.text.grapheme_iter(),
            ending: self.ending,
            done: false,
        }
    }
    
    
    /// Returns an iterator over the graphemes of the line
    pub fn grapheme_iter_at_index<'a>(&'a self, index: usize) -> LineGraphemeIter<'a> {
        LineGraphemeIter {
            graphemes: self.text.grapheme_iter_at_index(index),
            ending: self.ending,
            done: false,
        }
    }
}


/// Represents one of the valid Unicode line endings.
/// Also acts as an index into `LINE_ENDINGS`.
#[derive(PartialEq, Copy)]
pub enum LineEnding {
    None = 0,  // No line ending
    CRLF = 1,  // CarriageReturn followed by LineFeed
    LF = 2,    // U+000A -- LineFeed
    VT = 3,    // U+000B -- VerticalTab
    FF = 4,    // U+000C -- FormFeed
    CR = 5,    // U+000D -- CarriageReturn
    NEL = 6,   // U+0085 -- NextLine
    LS = 7,    // U+2028 -- Line Separator
    PS = 8,    // U+2029 -- ParagraphSeparator
}

pub fn str_to_line_ending(g: &str) -> LineEnding {
    match g {
        //==============
        // Line endings
        //==============
        
        // CRLF
        "\u{000D}\u{000A}" => {
            return LineEnding::CRLF;
        },
        
        // LF
        "\u{000A}" => {
            return LineEnding::LF;
        },
        
        // VT
        "\u{000B}" => {
            return LineEnding::VT;
        },
        
        // FF
        "\u{000C}" => {
            return LineEnding::FF;
        },
        
        // CR
        "\u{000D}" => {
            return LineEnding::CR;
        },
        
        // NEL
        "\u{0085}" => {
            return LineEnding::NEL;
        },
        
        // LS
        "\u{2028}" => {
            return LineEnding::LS;
        },
        
        // PS
        "\u{2029}" => {
            return LineEnding::PS;
        },
        
        // Not a line ending
        _ => {
            return LineEnding::None;
        }
    }
}

pub fn line_ending_to_str(ending: LineEnding) -> &'static str {
    LINE_ENDINGS[ending as usize]
}

/// An array of string literals corresponding to the possible
/// unicode line endings.
pub const LINE_ENDINGS: [&'static str; 9] = ["",
                          "\u{000D}\u{000A}",
                          "\u{000A}",
                          "\u{000B}",
                          "\u{000C}",
                          "\u{000D}",
                          "\u{0085}",
                          "\u{2028}",
                          "\u{2029}"
];


/// An iterator over the graphemes of a Line
pub struct LineGraphemeIter<'a> {
    graphemes: RopeGraphemeIter<'a>,
    ending: LineEnding,
    done: bool,
}

impl<'a> LineGraphemeIter<'a> {
    pub fn skip_graphemes(&mut self, n: usize) {
        for _ in range(0, n) {
            if let None = self.next() {
                break;
            }
        }
    }
}

impl<'a> Iterator for LineGraphemeIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        if self.done {
            return None;
        }
        else {
            let g = self.graphemes.next();
            if let Some(_) = g {
                return g;
            }
            else {
                self.done = true;
                
                if self.ending == LineEnding::None {
                    return None;
                }
                else {
                    return Some(LINE_ENDINGS[self.ending as usize]);
                }
            }
        }
    }
}




//=========================================================================
// Line tests
//=========================================================================

#[cfg(test)]
mod tests {
    use super::{Line, LineEnding, LineGraphemeIter};
    const TAB_WIDTH: usize = 4;


    #[test]
    fn new_text_line() {
        let tl = Line::new();
        
        assert_eq!(tl.text.grapheme_count(), 0);
        assert!(tl.ending == LineEnding::None);
    }
    
    #[test]
    fn new_text_line_from_str() {
        let tl = Line::new_from_str("Hello!");
        
        assert_eq!(tl.text.grapheme_count(), 6);
        assert!(&tl.text[0] == "H");
        assert!(&tl.text[1] == "e");
        assert!(&tl.text[2] == "l");
        assert!(&tl.text[3] == "l");
        assert!(&tl.text[4] == "o");
        assert!(&tl.text[5] == "!");
        assert!(tl.ending == LineEnding::None);
    }
    
    #[test]
    fn new_text_line_from_empty_str() {
        let tl = Line::new_from_str("");
        
        assert_eq!(tl.text.grapheme_count(), 0);
        assert!(tl.ending == LineEnding::None);
    }
    
    #[test]
    fn new_text_line_from_str_with_lf() {
        let tl = Line::new_from_str("Hello!\n");
        
        assert_eq!(tl.text.grapheme_count(), 6);
        assert!(&tl.text[0] == "H");
        assert!(&tl.text[1] == "e");
        assert!(&tl.text[2] == "l");
        assert!(&tl.text[3] == "l");
        assert!(&tl.text[4] == "o");
        assert!(&tl.text[5] == "!");
        assert!(tl.ending == LineEnding::LF);
    }
    
    #[test]
    fn new_text_line_from_str_with_crlf() {
        let tl = Line::new_from_str("Hello!\r\n");
        
        assert_eq!(tl.text.grapheme_count(), 6);
        assert!(&tl.text[0] == "H");
        assert!(&tl.text[1] == "e");
        assert!(&tl.text[2] == "l");
        assert!(&tl.text[3] == "l");
        assert!(&tl.text[4] == "o");
        assert!(&tl.text[5] == "!");
        assert!(tl.ending == LineEnding::CRLF);
    }
    
    #[test]
    fn new_text_line_from_str_with_crlf_and_too_long() {
        let tl = Line::new_from_str("Hello!\r\nLa la la la");
        
        assert_eq!(tl.text.grapheme_count(), 6);
        assert!(&tl.text[0] == "H");
        assert!(&tl.text[1] == "e");
        assert!(&tl.text[2] == "l");
        assert!(&tl.text[3] == "l");
        assert!(&tl.text[4] == "o");
        assert!(&tl.text[5] == "!");
        assert!(tl.ending == LineEnding::CRLF);
    }
    
    #[test]
    fn new_text_line_from_string_unchecked() {
        let s = String::from_str("Hello!");
        
        let tl = Line::new_from_string_unchecked(s);
        
        assert_eq!(tl.text.grapheme_count(), 6);
        assert!(&tl.text[0] == "H");
        assert!(&tl.text[1] == "e");
        assert!(&tl.text[2] == "l");
        assert!(&tl.text[3] == "l");
        assert!(&tl.text[4] == "o");
        assert!(&tl.text[5] == "!");
        assert!(tl.ending == LineEnding::None);
    }
    
    #[test]
    fn new_text_line_from_string_unchecked_with_lf() {
        let s = String::from_str("Hello!\u{000A}");
        
        let tl = Line::new_from_string_unchecked(s);
        
        assert_eq!(tl.text.grapheme_count(), 6);
        assert!(&tl.text[0] == "H");
        assert!(&tl.text[1] == "e");
        assert!(&tl.text[2] == "l");
        assert!(&tl.text[3] == "l");
        assert!(&tl.text[4] == "o");
        assert!(&tl.text[5] == "!");
        assert!(tl.ending == LineEnding::LF);
    }
    
    #[test]
    fn new_text_line_from_string_unchecked_with_crlf() {
        let s = String::from_str("Hello!\u{000D}\u{000A}");
        
        let tl = Line::new_from_string_unchecked(s);
        
        assert_eq!(tl.text.grapheme_count(), 6);
        assert!(&tl.text[0] == "H");
        assert!(&tl.text[1] == "e");
        assert!(&tl.text[2] == "l");
        assert!(&tl.text[3] == "l");
        assert!(&tl.text[4] == "o");
        assert!(&tl.text[5] == "!");
        assert!(tl.ending == LineEnding::CRLF);
    }
    
    #[test]
    fn new_text_line_from_string_unchecked_with_ls() {
        let s = String::from_str("Hello!\u{2028}");
        
        let tl = Line::new_from_string_unchecked(s);
        
        assert_eq!(tl.text.grapheme_count(), 6);
        assert!(&tl.text[0] == "H");
        assert!(&tl.text[1] == "e");
        assert!(&tl.text[2] == "l");
        assert!(&tl.text[3] == "l");
        assert!(&tl.text[4] == "o");
        assert!(&tl.text[5] == "!");
        assert!(tl.ending == LineEnding::LS);
    }
    
    #[test]
    fn text_line_insert_text() {
        let mut tl = Line::new_from_str("Hello!\r\n");
        
        tl.insert_text(" world", 5);
        
        assert_eq!(tl.text.grapheme_count(), 12);
        assert!(&tl.text[0] == "H");
        assert!(&tl.text[1] == "e");
        assert!(&tl.text[2] == "l");
        assert!(&tl.text[3] == "l");
        assert!(&tl.text[4] == "o");
        assert!(&tl.text[5] == " ");
        assert!(&tl.text[6] == "w");
        assert!(&tl.text[7] == "o");
        assert!(&tl.text[8] == "r");
        assert!(&tl.text[9] == "l");
        assert!(&tl.text[10] == "d");
        assert!(&tl.text[11] == "!");
        assert!(tl.ending == LineEnding::CRLF);
    }
    
    #[test]
    fn text_line_append_text() {
        let mut tl = Line::new_from_str("Hello\r\n");
        
        tl.append_text(" world!");
        
        assert_eq!(tl.text.grapheme_count(), 12);
        assert!(&tl.text[0] == "H");
        assert!(&tl.text[1] == "e");
        assert!(&tl.text[2] == "l");
        assert!(&tl.text[3] == "l");
        assert!(&tl.text[4] == "o");
        assert!(&tl.text[5] == " ");
        assert!(&tl.text[6] == "w");
        assert!(&tl.text[7] == "o");
        assert!(&tl.text[8] == "r");
        assert!(&tl.text[9] == "l");
        assert!(&tl.text[10] == "d");
        assert!(&tl.text[11] == "!");
        assert!(tl.ending == LineEnding::CRLF);
    }
    
    #[test]
    fn text_line_remove_text() {
        let mut tl = Line::new_from_str("Hello world!\r\n");
        
        tl.remove_text(5, 11);
        
        assert_eq!(tl.text.grapheme_count(), 6);
        assert!(&tl.text[0] == "H");
        assert!(&tl.text[1] == "e");
        assert!(&tl.text[2] == "l");
        assert!(&tl.text[3] == "l");
        assert!(&tl.text[4] == "o");
        assert!(&tl.text[5] == "!");
        assert!(tl.ending == LineEnding::CRLF);
    }
    
    #[test]
    fn text_line_split() {
        let mut tl1 = Line::new_from_str("Hello world!\r\n");
        
        let tl2 = tl1.split(LineEnding::LF, 5);
        
        assert_eq!(tl1.text.grapheme_count(), 5);
        assert!(&tl1.text[0] == "H");
        assert!(&tl1.text[1] == "e");
        assert!(&tl1.text[2] == "l");
        assert!(&tl1.text[3] == "l");
        assert!(&tl1.text[4] == "o");
        assert!(tl1.ending == LineEnding::LF);
        
        assert_eq!(tl2.text.grapheme_count(), 7);
        assert!(&tl2.text[0] == " ");
        assert!(&tl2.text[1] == "w");
        assert!(&tl2.text[2] == "o");
        assert!(&tl2.text[3] == "r");
        assert!(&tl2.text[4] == "l");
        assert!(&tl2.text[5] == "d");
        assert!(&tl2.text[6] == "!");
        assert!(tl2.ending == LineEnding::CRLF);
    }
    
    #[test]
    fn text_line_split_beginning() {
        let mut tl1 = Line::new_from_str("Hello!\r\n");
        
        let tl2 = tl1.split(LineEnding::LF, 0);
        
        assert_eq!(tl1.text.grapheme_count(), 0);
        assert!(tl1.ending == LineEnding::LF);
        
        assert_eq!(tl2.text.grapheme_count(), 6);
        assert!(&tl2.text[0] == "H");
        assert!(&tl2.text[1] == "e");
        assert!(&tl2.text[2] == "l");
        assert!(&tl2.text[3] == "l");
        assert!(&tl2.text[4] == "o");
        assert!(&tl2.text[5] == "!");
        assert!(tl2.ending == LineEnding::CRLF);
    }
    
    
    //=========================================================================
    // LineGraphemeIter tests
    //=========================================================================
    
    #[test]
    fn text_line_grapheme_iter() {
        let tl = Line::new_from_str("Hello!");
        let mut iter = tl.grapheme_iter();
        
        assert!(iter.next() == Some("H"));
        assert!(iter.next() == Some("e"));
        assert!(iter.next() == Some("l"));
        assert!(iter.next() == Some("l"));
        assert!(iter.next() == Some("o"));
        assert!(iter.next() == Some("!"));
        assert!(iter.next() == None);
    }
    
    #[test]
    fn text_line_grapheme_iter_with_lf() {
        let tl = Line::new_from_str("Hello!\n");
        let mut iter = tl.grapheme_iter();
        
        assert!(iter.next() == Some("H"));
        assert!(iter.next() == Some("e"));
        assert!(iter.next() == Some("l"));
        assert!(iter.next() == Some("l"));
        assert!(iter.next() == Some("o"));
        assert!(iter.next() == Some("!"));
        assert!(iter.next() == Some("\n"));
        assert!(iter.next() == None);
    }
    
    #[test]
    fn text_line_grapheme_iter_with_crlf() {
        let tl = Line::new_from_str("Hello!\r\n");
        let mut iter = tl.grapheme_iter();
        
        assert!(iter.next() == Some("H"));
        assert!(iter.next() == Some("e"));
        assert!(iter.next() == Some("l"));
        assert!(iter.next() == Some("l"));
        assert!(iter.next() == Some("o"));
        assert!(iter.next() == Some("!"));
        assert!(iter.next() == Some("\r\n"));
        assert!(iter.next() == None);
    }
    
    #[test]
    fn text_line_grapheme_iter_at_index() {
        let tl = Line::new_from_str("Hello!");
        let mut iter = tl.grapheme_iter_at_index(2);
        
        assert!(iter.next() == Some("l"));
        assert!(iter.next() == Some("l"));
        assert!(iter.next() == Some("o"));
        assert!(iter.next() == Some("!"));
        assert!(iter.next() == None);
    }
    
    #[test]
    fn text_line_grapheme_iter_at_index_past_end() {
        let tl = Line::new_from_str("Hello!");
        let mut iter = tl.grapheme_iter_at_index(10);
        
        assert!(iter.next() == None);
    }
    
    #[test]
    fn text_line_grapheme_iter_at_index_at_lf() {
        let tl = Line::new_from_str("Hello!\n");
        let mut iter = tl.grapheme_iter_at_index(6);
        
        assert!(iter.next() == Some("\n"));
        assert!(iter.next() == None);
    }
    
}