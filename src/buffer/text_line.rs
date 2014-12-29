#![allow(dead_code)]

use std::mem;
use std::str::Graphemes;
use string_utils::{grapheme_pos_to_byte_pos, is_line_ending};


/// A single line of text
pub struct TextLine {
    text: Vec<u8>, // The text data, stored as UTF8
    ending: LineEnding, // The type of line ending, if any
}


impl TextLine {
    /// Creates a new empty TextLine
    pub fn new() -> TextLine {
        TextLine {
            text: Vec::new(),
            ending: LineEnding::None,
        }
    }
    
    
    /// Creates a new TextLine from a str.
    pub fn new_from_str(text: &str) -> TextLine {
        // Initialize TextLine
        let mut tl = TextLine {
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
                panic!("TextLine::insert_text(): line ending in inserted text.");
            }
            
            for b in g.bytes() {
                self.text[i] = b;
                i += 1
            }
        }
    }
    
    
    /// Returns an iterator over the graphemes of the line
    pub fn grapheme_iter<'a>(&'a self) -> TextLineIter<'a> {
        TextLineIter {
            graphemes: self.as_str().graphemes(true),
            ending: self.ending,
            done: false,
        }
    }
    
    
    /// Returns an iterator over the graphemes of the line
    pub fn grapheme_iter_at_index<'a>(&'a self, index: uint) -> TextLineIter<'a> {
        let temp: &str = unsafe{mem::transmute(self.text.as_slice())};
        
        let mut iter = TextLineIter {
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


/// An iterator over the graphemes of a TextLine
pub struct TextLineIter<'a> {
    graphemes: Graphemes<'a>,
    ending: LineEnding,
    done: bool,
}

impl<'a> Iterator<&'a str> for TextLineIter<'a> {
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
// TextLine tests
//=========================================================================

#[test]
fn new_text_line() {
    let tl = TextLine::new();
    
    assert!(tl.text.len() == 0);
    assert!(tl.ending == LineEnding::None);
}

#[test]
fn new_text_line_from_str() {
    let tl = TextLine::new_from_str("Hello!");
    
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
    let tl = TextLine::new_from_str("");
    
    assert!(tl.text.len() == 0);
    assert!(tl.ending == LineEnding::None);
}

#[test]
fn new_text_line_from_str_with_lf() {
    let tl = TextLine::new_from_str("Hello!\n");
    
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
    let tl = TextLine::new_from_str("Hello!\r\n");
    
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
    let tl = TextLine::new_from_str("Hello!\r\nLa la la la");
    
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
    let mut tl = TextLine::new_from_str("Hello!\r\n");
    
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


//=========================================================================
// TextLineIter tests
//=========================================================================

#[test]
fn text_line_grapheme_iter() {
    let tl = TextLine::new_from_str("Hello!");
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
    let tl = TextLine::new_from_str("Hello!\n");
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
    let tl = TextLine::new_from_str("Hello!\r\n");
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
    let tl = TextLine::new_from_str("Hello!");
    let mut iter = tl.grapheme_iter_at_index(2);
    
    assert!(iter.next() == Some("l"));
    assert!(iter.next() == Some("l"));
    assert!(iter.next() == Some("o"));
    assert!(iter.next() == Some("!"));
    assert!(iter.next() == None);
}

#[test]
fn text_line_grapheme_iter_at_index_past_end() {
    let tl = TextLine::new_from_str("Hello!");
    let mut iter = tl.grapheme_iter_at_index(10);
    
    assert!(iter.next() == None);
}

#[test]
fn text_line_grapheme_iter_at_index_at_lf() {
    let tl = TextLine::new_from_str("Hello!\n");
    let mut iter = tl.grapheme_iter_at_index(6);
    
    assert!(iter.next() == Some("\n"));
    assert!(iter.next() == None);
}