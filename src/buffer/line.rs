#![allow(dead_code)]

use std::mem;
use std::str::Graphemes;
use string_utils::{grapheme_count, grapheme_pos_to_byte_pos, is_line_ending};


/// A single line of text
pub struct Line {
    text: Vec<u8>, // The text data, stored as UTF8
    pub ending: LineEnding, // The type of line ending, if any
}


impl Line {
    /// Creates a new empty Line
    pub fn new() -> Line {
        Line {
            text: Vec::new(),
            ending: LineEnding::None,
        }
    }
    
    
    /// Creates a new Line from a str.
    pub fn new_from_str(text: &str) -> Line {
        // Initialize Line
        let mut tl = Line {
            text: Vec::with_capacity(text.len()),
            ending: LineEnding::None,
        };
        
        // Copy text data, stopping on a line ending if any is found
        for g in text.graphemes(true) {        
            match g {
                //==============
                // Line endings
                //==============
                
                // CRLF
                "\u{000D}\u{000A}" => {
                    tl.ending = LineEnding::CRLF;
                    break;
                },
                
                // LF
                "\u{000A}" => {
                    tl.ending = LineEnding::LF;
                    break;
                },
                
                // VT
                "\u{000B}" => {
                    tl.ending = LineEnding::VT;
                    break;
                },
                
                // FF
                "\u{000C}" => {
                    tl.ending = LineEnding::FF;
                    break;
                },
                
                // CR
                "\u{000D}" => {
                    tl.ending = LineEnding::CR;
                    break;
                },
                
                // NEL
                "\u{0085}" => {
                    tl.ending = LineEnding::NEL;
                    break;
                },
                
                // LS
                "\u{2028}" => {
                    tl.ending = LineEnding::LS;
                    break;
                },
                
                // PS
                "\u{2029}" => {
                    tl.ending = LineEnding::PS;
                    break;
                },
                
                //==================
                // Other characters
                //==================
                
                _ => {
                    for b in g.bytes() {
                        tl.text.push(b);
                    }
                }
            }
        }
        
        // Done!
        return tl;
    }
    
    
    /// Returns the total number of unicode graphemes in the line
    pub fn grapheme_count(&self) -> uint {
        let mut count = grapheme_count(self.as_str());
        match self.ending {
            LineEnding::None => {},
            _ => {count += 1;}
        }
        return count;
    }
    
    
    /// Returns the total number of unicode graphemes in the line,
    /// not counting the line ending grapheme, if any.
    pub fn grapheme_count_sans_line_ending(&self) -> uint {
        grapheme_count(self.as_str())
    }
    
    
    /// Returns an immutable string slice into the text block's memory
    pub fn as_str<'a>(&'a self) -> &'a str {
        unsafe {
            mem::transmute(self.text.as_slice())
        }
    }
    
    
    /// Inserts `text` at grapheme index `pos`.
    /// NOTE: panics if it encounters a line ending in the text.
    pub fn insert_text(&mut self, text: &str, pos: uint) {
        // Find insertion position in bytes
        let byte_pos = grapheme_pos_to_byte_pos(self.as_str(), pos);

        // Grow data size        
        self.text.grow(text.len(), 0);
        
        // Move old bytes forward
        let mut from = self.text.len() - text.len();
        let mut to = self.text.len();
        while from > byte_pos {
            from -= 1;
            to -= 1;
            
            self.text[to] = self.text[from];
        }
        
        // Copy new bytes in
        let mut i = byte_pos;
        for g in text.graphemes(true) {
            if is_line_ending(g) {
                panic!("Line::insert_text(): line ending in inserted text.");
            }
            
            for b in g.bytes() {
                self.text[i] = b;
                i += 1
            }
        }
    }
    
    
    /// Remove the text between grapheme positions 'pos_a' and 'pos_b'.
    pub fn remove_text(&mut self, pos_a: uint, pos_b: uint) {
        // Bounds checks
        if pos_a > pos_b {
            panic!("Line::remove_text(): pos_a must be less than or equal to pos_b.");
        }
        
        // Find removal positions in bytes
        let byte_pos_a = grapheme_pos_to_byte_pos(self.as_str(), pos_a);
        let byte_pos_b = grapheme_pos_to_byte_pos(self.as_str(), pos_b);
        
        // Move bytes to fill in the gap left by the removed bytes
        let mut from = byte_pos_b;
        let mut to = byte_pos_a;
        while from < self.text.len() {
            self.text[to] = self.text[from];
            
            from += 1;
            to += 1;
        }
        
        // Remove data from the end
        let final_text_size = self.text.len() + byte_pos_a - byte_pos_b;
        self.text.truncate(final_text_size);
    }
    
    
    /// Insert a line break into the line, splitting it into two.
    /// This line stays as the first part of the split.  The second
    /// part is returned.
    pub fn split(&mut self, ending: LineEnding, pos: uint) -> Line {
        let mut other = Line::new();
        
        // Inserting at very beginning: special cased for efficiency
        if pos == 0 {
            mem::swap(self, &mut other);
            self.ending = ending;
        }
        // Otherwise, general case
        else {
            // Find the byte index to split at
            let byte_pos = grapheme_pos_to_byte_pos(self.as_str(), pos);
            
            // Copy the elements after the split index to the second line
            other.text.push_all(self.text.slice_from_or_fail(&byte_pos));
            
            // Truncate the first line
            self.text.truncate(byte_pos);
            
            // Set the line endings appropriately
            other.ending = self.ending;
            self.ending = ending;
        }
        
        return other;
    }
    
    
    /// Returns an iterator over the graphemes of the line
    pub fn grapheme_iter<'a>(&'a self) -> LineGraphemeIter<'a> {
        LineGraphemeIter {
            graphemes: self.as_str().graphemes(true),
            ending: self.ending,
            done: false,
        }
    }
    
    
    /// Returns an iterator over the graphemes of the line
    pub fn grapheme_iter_at_index<'a>(&'a self, index: uint) -> LineGraphemeIter<'a> {
        let temp: &str = unsafe{mem::transmute(self.text.as_slice())};
        
        let mut iter = LineGraphemeIter {
            graphemes: temp.graphemes(true),
            ending: self.ending,
            done: false,
        };
        
        for _ in range(0, index) {
            iter.next();
        }
        
        return iter;
    }
}


/// Represents one of the valid Unicode line endings.
/// Also acts as an index into `LINE_ENDINGS`.
#[deriving(PartialEq, Copy)]
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

/// An array of string literals corresponding to the possible
/// unicode line endings.
pub const LINE_ENDINGS: [&'static str, ..9] = ["",
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
    graphemes: Graphemes<'a>,
    ending: LineEnding,
    done: bool,
}

impl<'a> Iterator<&'a str> for LineGraphemeIter<'a> {
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
                    return Some(LINE_ENDINGS[self.ending as uint]);
                }
            }
        }
    }
}




//=========================================================================
// Line tests
//=========================================================================

#[test]
fn new_text_line() {
    let tl = Line::new();
    
    assert!(tl.text.len() == 0);
    assert!(tl.ending == LineEnding::None);
}

#[test]
fn new_text_line_from_str() {
    let tl = Line::new_from_str("Hello!");
    
    assert!(tl.text.len() == 6);
    assert!(tl.text[0] == ('H' as u8));
    assert!(tl.text[1] == ('e' as u8));
    assert!(tl.text[2] == ('l' as u8));
    assert!(tl.text[3] == ('l' as u8));
    assert!(tl.text[4] == ('o' as u8));
    assert!(tl.text[5] == ('!' as u8));
    assert!(tl.ending == LineEnding::None);
}

#[test]
fn new_text_line_from_empty_str() {
    let tl = Line::new_from_str("");
    
    assert!(tl.text.len() == 0);
    assert!(tl.ending == LineEnding::None);
}

#[test]
fn new_text_line_from_str_with_lf() {
    let tl = Line::new_from_str("Hello!\n");
    
    assert!(tl.text.len() == 6);
    assert!(tl.text[0] == ('H' as u8));
    assert!(tl.text[1] == ('e' as u8));
    assert!(tl.text[2] == ('l' as u8));
    assert!(tl.text[3] == ('l' as u8));
    assert!(tl.text[4] == ('o' as u8));
    assert!(tl.text[5] == ('!' as u8));
    assert!(tl.ending == LineEnding::LF);
}

#[test]
fn new_text_line_from_str_with_crlf() {
    let tl = Line::new_from_str("Hello!\r\n");
    
    assert!(tl.text.len() == 6);
    assert!(tl.text[0] == ('H' as u8));
    assert!(tl.text[1] == ('e' as u8));
    assert!(tl.text[2] == ('l' as u8));
    assert!(tl.text[3] == ('l' as u8));
    assert!(tl.text[4] == ('o' as u8));
    assert!(tl.text[5] == ('!' as u8));
    assert!(tl.ending == LineEnding::CRLF);
}

#[test]
fn new_text_line_from_str_with_crlf_and_too_long() {
    let tl = Line::new_from_str("Hello!\r\nLa la la la");
    
    assert!(tl.text.len() == 6);
    assert!(tl.text[0] == ('H' as u8));
    assert!(tl.text[1] == ('e' as u8));
    assert!(tl.text[2] == ('l' as u8));
    assert!(tl.text[3] == ('l' as u8));
    assert!(tl.text[4] == ('o' as u8));
    assert!(tl.text[5] == ('!' as u8));
    assert!(tl.ending == LineEnding::CRLF);
}

#[test]
fn text_line_insert_text() {
    let mut tl = Line::new_from_str("Hello!\r\n");
    
    tl.insert_text(" world", 5);
    
    assert!(tl.text.len() == 12);
    assert!(tl.text[0] == ('H' as u8));
    assert!(tl.text[1] == ('e' as u8));
    assert!(tl.text[2] == ('l' as u8));
    assert!(tl.text[3] == ('l' as u8));
    assert!(tl.text[4] == ('o' as u8));
    assert!(tl.text[5] == (' ' as u8));
    assert!(tl.text[6] == ('w' as u8));
    assert!(tl.text[7] == ('o' as u8));
    assert!(tl.text[8] == ('r' as u8));
    assert!(tl.text[9] == ('l' as u8));
    assert!(tl.text[10] == ('d' as u8));
    assert!(tl.text[11] == ('!' as u8));
    assert!(tl.ending == LineEnding::CRLF);
}

#[test]
fn text_line_remove_text() {
    let mut tl = Line::new_from_str("Hello world!\r\n");
    
    tl.remove_text(5, 11);
    
    assert!(tl.text.len() == 6);
    assert!(tl.text[0] == ('H' as u8));
    assert!(tl.text[1] == ('e' as u8));
    assert!(tl.text[2] == ('l' as u8));
    assert!(tl.text[3] == ('l' as u8));
    assert!(tl.text[4] == ('o' as u8));
    assert!(tl.text[5] == ('!' as u8));
    assert!(tl.ending == LineEnding::CRLF);
}

#[test]
fn text_line_split() {
    let mut tl1 = Line::new_from_str("Hello world!\r\n");
    
    let tl2 = tl1.split(LineEnding::LF, 5);
    
    assert!(tl1.text.len() == 5);
    assert!(tl1.text[0] == ('H' as u8));
    assert!(tl1.text[1] == ('e' as u8));
    assert!(tl1.text[2] == ('l' as u8));
    assert!(tl1.text[3] == ('l' as u8));
    assert!(tl1.text[4] == ('o' as u8));
    assert!(tl1.ending == LineEnding::LF);
    
    assert!(tl2.text.len() == 7);
    assert!(tl2.text[0] == (' ' as u8));
    assert!(tl2.text[1] == ('w' as u8));
    assert!(tl2.text[2] == ('o' as u8));
    assert!(tl2.text[3] == ('r' as u8));
    assert!(tl2.text[4] == ('l' as u8));
    assert!(tl2.text[5] == ('d' as u8));
    assert!(tl2.text[6] == ('!' as u8));
    assert!(tl2.ending == LineEnding::CRLF);
}

#[test]
fn text_line_split_beginning() {
    let mut tl1 = Line::new_from_str("Hello!\r\n");
    
    let tl2 = tl1.split(LineEnding::LF, 0);
    
    assert!(tl1.text.len() == 0);
    assert!(tl1.ending == LineEnding::LF);
    
    assert!(tl2.text.len() == 6);
    assert!(tl2.text[0] == ('H' as u8));
    assert!(tl2.text[1] == ('e' as u8));
    assert!(tl2.text[2] == ('l' as u8));
    assert!(tl2.text[3] == ('l' as u8));
    assert!(tl2.text[4] == ('o' as u8));
    assert!(tl2.text[5] == ('!' as u8));
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