#![allow(dead_code)]

use std::mem;

use self::node::{BufferNode, BufferNodeGraphemeIter, BufferNodeLineIter};
use self::line::{Line};
use string_utils::{is_line_ending};

mod line;
mod node;


//=============================================================
// Buffer
//=============================================================

/// A text buffer
pub struct Buffer {
    root: BufferNode,
}


impl Buffer {
    pub fn new() -> Buffer {
        Buffer {
            root: BufferNode::new()
        }
    }

    
    pub fn len(&self) -> uint {
        self.root.grapheme_count
    }

    
    pub fn line_count(&self) -> uint {
        self.root.line_count
    }
    
    
    pub fn get_line<'a>(&'a self, index: uint) -> &'a Line {
        if index >= self.line_count() {
            panic!("get_line(): index out of bounds.");
        }
        
        // NOTE: this can be done non-recursively, which would be more
        // efficient.  However, it seems likely to require unsafe code
        // if done that way.
        return self.root.get_line_recursive(index);
    }
    
    
    /// Removes the lines in line indices [line_a, line_b).
    pub fn remove_lines(&mut self, line_a: uint, line_b: uint) {
        // Nothing to do
        if line_a == line_b {
            return;
        }
        // Bounds error
        else if line_a > line_b {
            panic!("Buffer::remove_lines(): line_a must be less than or equal to line_b.");
        }
        // Bounds error
        else if line_b > self.line_count() {
            panic!("Buffer::remove_lines(): attempt to remove lines past the last line of text.");
        }
        // Complete removal of all lines
        else if line_a == 0 && line_b == self.root.line_count {
            let mut temp_node = BufferNode::new();
            mem::swap(&mut (self.root), &mut temp_node);
        }
        // All other cases
        else {
            self.root.remove_lines_recursive(line_a, line_b);
            self.root.set_last_line_ending_recursive();
        }
    }

    
    pub fn pos_2d_to_closest_1d(&self, pos: (uint, uint)) -> uint {
        return self.root.pos_2d_to_closest_1d_recursive(pos);
    }

    
    pub fn pos_1d_to_closest_2d(&self, pos: uint) -> (uint, uint) {
        return self.root.pos_1d_to_closest_2d_recursive(pos);
    }

    
    /// Insert 'text' at grapheme position 'pos'.
    pub fn insert_text(&mut self, text: &str, pos: uint) {
        self.root.insert_text(text, pos);
    }

    
    /// Remove the text between grapheme positions 'pos_a' and 'pos_b'.
    pub fn remove_text(&mut self, pos_a: uint, pos_b: uint) {
        // Nothing to do
        if pos_a == pos_b {
            return;
        }
        // Bounds error
        else if pos_a > pos_b {
            panic!("Buffer::remove_text(): pos_a must be less than or equal to pos_b.");
        }
        // Bounds error
        else if pos_b > self.len() {
            panic!("Buffer::remove_text(): attempt to remove text past the end of buffer.");
        }
        // Complete removal of all text
        else if pos_a == 0 && pos_b == self.root.grapheme_count {
            let mut temp_node = BufferNode::new();
            mem::swap(&mut (self.root), &mut temp_node);
        }
        // All other cases
        else {
            if self.root.remove_text_recursive(pos_a, pos_b, true) {
                panic!("Buffer::remove_text(): dangling left side remains.  This should never happen!");
            }
            self.root.set_last_line_ending_recursive();
        }
    }

    
    /// Creates an iterator at the first character
    pub fn grapheme_iter<'a>(&'a self) -> BufferGraphemeIter<'a> {
        BufferGraphemeIter {
            gi: self.root.grapheme_iter()
        }
    }
    
    
    /// Creates an iterator starting at the specified grapheme index.
    /// If the index is past the end of the text, then the iterator will
    /// return None on next().
    pub fn grapheme_iter_at_index<'a>(&'a self, index: uint) -> BufferGraphemeIter<'a> {
        BufferGraphemeIter {
            gi: self.root.grapheme_iter_at_index(index)
        }
    }
    
    
    pub fn line_iter<'a>(&'a self) -> BufferLineIter<'a> {
        BufferLineIter {
            li: self.root.line_iter()
        }
    }
    
    
    pub fn line_iter_at_index<'a>(&'a self, index: uint) -> BufferLineIter<'a> {
        BufferLineIter {
            li: self.root.line_iter_at_index(index)
        }
    }
    

}




//=============================================================
// Buffer iterators
//=============================================================

/// An iterator over a text buffer's graphemes
pub struct BufferGraphemeIter<'a> {
    gi: BufferNodeGraphemeIter<'a>,
}


impl<'a> BufferGraphemeIter<'a> {
    // Puts the iterator on the next line.
    // Returns true if there was a next line,
    // false if there wasn't.
    pub fn next_line(&mut self) -> bool {
        self.gi.next_line()
    }
    
    
    // Skips the iterator n graphemes ahead.
    // If it runs out of graphemes before reaching the desired skip count,
    // returns false.  Otherwise returns true.
    pub fn skip_graphemes(&mut self, n: uint) -> bool {
        self.gi.skip_graphemes(n)
    }
    
    pub fn skip_non_newline_graphemes(&mut self, n: uint) -> bool {
        let mut i: uint = 0;
        
        for g in self.gi {
            if is_line_ending(g) {
                return true;
            }
            
            i += 1;
            if i >= n {
                break;
            }
        }
        
        return false;
    }
}


impl<'a> Iterator<&'a str> for BufferGraphemeIter<'a> {
    fn next(&mut self) -> Option<&'a str> {
        self.gi.next()
    }
}


pub struct BufferLineIter<'a> {
    li: BufferNodeLineIter<'a>,
}


impl<'a> Iterator<&'a Line> for BufferLineIter<'a> {
    fn next(&mut self) -> Option<&'a Line> {
        self.li.next()
    }
}





//================================================================
// TESTS
//================================================================

#[test]
fn insert_text() {
    let mut buf = Buffer::new();
    
    buf.insert_text("Hello 世界!", 0);
    
    let mut iter = buf.grapheme_iter();
    
    assert!(buf.len() == 9);
    assert!(buf.root.line_count == 1);
    assert!(Some("H") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some("l") == iter.next());
    assert!(Some("l") == iter.next());
    assert!(Some("o") == iter.next());
    assert!(Some(" ") == iter.next());
    assert!(Some("世") == iter.next());
    assert!(Some("界") == iter.next());
    assert!(Some("!") == iter.next());
    assert!(None == iter.next());
}


#[test]
fn insert_text_with_newlines() {
    let mut buf = Buffer::new();
    
    buf.insert_text("Hello\n 世界\r\n!", 0);
    
    let mut iter = buf.grapheme_iter();
    
    assert!(buf.len() == 11);
    assert!(buf.root.line_count == 3);
    assert!(Some("H") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some("l") == iter.next());
    assert!(Some("l") == iter.next());
    assert!(Some("o") == iter.next());
    assert!(Some("\n") == iter.next());
    assert!(Some(" ") == iter.next());
    assert!(Some("世") == iter.next());
    assert!(Some("界") == iter.next());
    assert!(Some("\r\n") == iter.next());
    assert!(Some("!") == iter.next());
    assert!(None == iter.next());
}


#[test]
fn insert_text_in_non_empty_buffer_1() {
    let mut buf = Buffer::new();
    
    buf.insert_text("Hello\n 世界\r\n!", 0);
    buf.insert_text("Again ", 0);
    
    let mut iter = buf.grapheme_iter();
    
    assert!(buf.len() == 17);
    assert!(buf.root.line_count == 3);
    assert!(Some("A") == iter.next());
    assert!(Some("g") == iter.next());
    assert!(Some("a") == iter.next());
    assert!(Some("i") == iter.next());
    assert!(Some("n") == iter.next());
    assert!(Some(" ") == iter.next());
    assert!(Some("H") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some("l") == iter.next());
    assert!(Some("l") == iter.next());
    assert!(Some("o") == iter.next());
    assert!(Some("\n") == iter.next());
    assert!(Some(" ") == iter.next());
    assert!(Some("世") == iter.next());
    assert!(Some("界") == iter.next());
    assert!(Some("\r\n") == iter.next());
    assert!(Some("!") == iter.next());
    assert!(None == iter.next());
}


#[test]
fn insert_text_in_non_empty_buffer_2() {
    let mut buf = Buffer::new();
    
    buf.insert_text("Hello\n 世界\r\n!", 0);
    buf.insert_text(" again", 5);
    
    let mut iter = buf.grapheme_iter();
    
    assert!(buf.len() == 17);
    assert!(buf.root.line_count == 3);
    assert!(Some("H") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some("l") == iter.next());
    assert!(Some("l") == iter.next());
    assert!(Some("o") == iter.next());
    assert!(Some(" ") == iter.next());
    assert!(Some("a") == iter.next());
    assert!(Some("g") == iter.next());
    assert!(Some("a") == iter.next());
    assert!(Some("i") == iter.next());
    assert!(Some("n") == iter.next());
    assert!(Some("\n") == iter.next());
    assert!(Some(" ") == iter.next());
    assert!(Some("世") == iter.next());
    assert!(Some("界") == iter.next());
    assert!(Some("\r\n") == iter.next());
    assert!(Some("!") == iter.next());
    assert!(None == iter.next());
}


#[test]
fn insert_text_in_non_empty_buffer_3() {
    let mut buf = Buffer::new();
    
    buf.insert_text("Hello\n 世界\r\n!", 0);
    buf.insert_text("again", 6);
    
    let mut iter = buf.grapheme_iter();
    
    assert!(buf.len() == 16);
    assert!(buf.root.line_count == 3);
    assert!(Some("H") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some("l") == iter.next());
    assert!(Some("l") == iter.next());
    assert!(Some("o") == iter.next());
    assert!(Some("\n") == iter.next());
    assert!(Some("a") == iter.next());
    assert!(Some("g") == iter.next());
    assert!(Some("a") == iter.next());
    assert!(Some("i") == iter.next());
    assert!(Some("n") == iter.next());
    assert!(Some(" ") == iter.next());
    assert!(Some("世") == iter.next());
    assert!(Some("界") == iter.next());
    assert!(Some("\r\n") == iter.next());
    assert!(Some("!") == iter.next());
    assert!(None == iter.next());
}


#[test]
fn insert_text_in_non_empty_buffer_4() {
    let mut buf = Buffer::new();
    
    buf.insert_text("Hello\n 世界\r\n!", 0);
    buf.insert_text("again", 11);
    
    let mut iter = buf.grapheme_iter();
    
    assert!(buf.len() == 16);
    assert!(buf.root.line_count == 3);
    assert!(Some("H") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some("l") == iter.next());
    assert!(Some("l") == iter.next());
    assert!(Some("o") == iter.next());
    assert!(Some("\n") == iter.next());
    assert!(Some(" ") == iter.next());
    assert!(Some("世") == iter.next());
    assert!(Some("界") == iter.next());
    assert!(Some("\r\n") == iter.next());
    assert!(Some("!") == iter.next());
    assert!(Some("a") == iter.next());
    assert!(Some("g") == iter.next());
    assert!(Some("a") == iter.next());
    assert!(Some("i") == iter.next());
    assert!(Some("n") == iter.next());
    assert!(None == iter.next());
}


#[test]
fn insert_text_in_non_empty_buffer_5() {
    let mut buf = Buffer::new();
    
    buf.insert_text("Hello\n 世界\r\n!", 0);
    buf.insert_text("again", 2);
    
    let mut iter = buf.grapheme_iter();
    
    assert!(buf.len() == 16);
    assert!(buf.root.line_count == 3);
    assert!(Some("H") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some("a") == iter.next());
    assert!(Some("g") == iter.next());
    assert!(Some("a") == iter.next());
    assert!(Some("i") == iter.next());
    assert!(Some("n") == iter.next());
    assert!(Some("l") == iter.next());
    assert!(Some("l") == iter.next());
    assert!(Some("o") == iter.next());
    assert!(Some("\n") == iter.next());
    assert!(Some(" ") == iter.next());
    assert!(Some("世") == iter.next());
    assert!(Some("界") == iter.next());
    assert!(Some("\r\n") == iter.next());
    assert!(Some("!") == iter.next());
    
    assert!(None == iter.next());
}


#[test]
fn insert_text_in_non_empty_buffer_6() {
    let mut buf = Buffer::new();
    
    buf.insert_text("Hello\n 世界\r\n!", 0);
    buf.insert_text("again", 8);
    
    let mut iter = buf.grapheme_iter();
    
    assert!(buf.len() == 16);
    assert!(buf.root.line_count == 3);
    assert!(Some("H") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some("l") == iter.next());
    assert!(Some("l") == iter.next());
    assert!(Some("o") == iter.next());
    assert!(Some("\n") == iter.next());
    assert!(Some(" ") == iter.next());
    assert!(Some("世") == iter.next());
    assert!(Some("a") == iter.next());
    assert!(Some("g") == iter.next());
    assert!(Some("a") == iter.next());
    assert!(Some("i") == iter.next());
    assert!(Some("n") == iter.next());
    assert!(Some("界") == iter.next());
    assert!(Some("\r\n") == iter.next());
    assert!(Some("!") == iter.next());
    
    assert!(None == iter.next());
}


#[test]
fn insert_text_in_non_empty_buffer_7() {
    let mut buf = Buffer::new();
    
    buf.insert_text("Hello\n 世界\r\n!", 0);
    buf.insert_text("\nag\n\nain\n", 2);
    
    let mut iter = buf.grapheme_iter();
    
    assert!(buf.len() == 20);
    assert!(buf.root.line_count == 7);
    assert!(Some("H") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some("\n") == iter.next());
    assert!(Some("a") == iter.next());
    assert!(Some("g") == iter.next());
    assert!(Some("\n") == iter.next());
    assert!(Some("\n") == iter.next());
    assert!(Some("a") == iter.next());
    assert!(Some("i") == iter.next());
    assert!(Some("n") == iter.next());
    assert!(Some("\n") == iter.next());
    assert!(Some("l") == iter.next());
    assert!(Some("l") == iter.next());
    assert!(Some("o") == iter.next());
    assert!(Some("\n") == iter.next());
    assert!(Some(" ") == iter.next());
    assert!(Some("世") == iter.next());
    assert!(Some("界") == iter.next());
    assert!(Some("\r\n") == iter.next());
    assert!(Some("!") == iter.next());
    
    assert!(None == iter.next());
}


#[test]
fn remove_text_1() {
    let mut buf = Buffer::new();
    
    buf.insert_text("Hi\nthere\npeople\nof\nthe\nworld!", 0);
    assert!(buf.len() == 29);
    assert!(buf.root.line_count == 6);
    
    buf.remove_text(0, 3);
    
    let mut iter = buf.grapheme_iter();
    
    assert!(buf.len() == 26);
    assert!(buf.root.line_count == 5);
    assert!(Some("t") == iter.next());
    assert!(Some("h") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some("r") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some("\n") == iter.next());
    assert!(Some("p") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some("o") == iter.next());
    assert!(Some("p") == iter.next());
    assert!(Some("l") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some("\n") == iter.next());
    assert!(Some("o") == iter.next());
    assert!(Some("f") == iter.next());
    assert!(Some("\n") == iter.next());
    assert!(Some("t") == iter.next());
    assert!(Some("h") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some("\n") == iter.next());
    assert!(Some("w") == iter.next());
    assert!(Some("o") == iter.next());
    assert!(Some("r") == iter.next());
    assert!(Some("l") == iter.next());
    assert!(Some("d") == iter.next());
    assert!(Some("!") == iter.next());
    assert!(None == iter.next());
}


#[test]
fn remove_text_2() {
    let mut buf = Buffer::new();
    
    buf.insert_text("Hi\nthere\npeople\nof\nthe\nworld!", 0);
    assert!(buf.len() == 29);
    assert!(buf.root.line_count == 6);
    
    buf.remove_text(0, 12);
    
    let mut iter = buf.grapheme_iter();
    
    assert!(buf.len() == 17);
    assert!(buf.root.line_count == 4);
    assert!(Some("p") == iter.next());
    assert!(Some("l") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some("\n") == iter.next());
    assert!(Some("o") == iter.next());
    assert!(Some("f") == iter.next());
    assert!(Some("\n") == iter.next());
    assert!(Some("t") == iter.next());
    assert!(Some("h") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some("\n") == iter.next());
    assert!(Some("w") == iter.next());
    assert!(Some("o") == iter.next());
    assert!(Some("r") == iter.next());
    assert!(Some("l") == iter.next());
    assert!(Some("d") == iter.next());
    assert!(Some("!") == iter.next());
    assert!(None == iter.next());
}


#[test]
fn remove_text_3() {
    let mut buf = Buffer::new();
    
    buf.insert_text("Hi\nthere\npeople\nof\nthe\nworld!", 0);
    assert!(buf.len() == 29);
    assert!(buf.root.line_count == 6);
    
    buf.remove_text(5, 17);
    
    let mut iter = buf.grapheme_iter();
    
    assert!(buf.len() == 17);
    assert!(buf.root.line_count == 4);
    assert!(Some("H") == iter.next());
    assert!(Some("i") == iter.next());
    assert!(Some("\n") == iter.next());
    assert!(Some("t") == iter.next());
    assert!(Some("h") == iter.next());
    assert!(Some("f") == iter.next());
    assert!(Some("\n") == iter.next());
    assert!(Some("t") == iter.next());
    assert!(Some("h") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some("\n") == iter.next());
    assert!(Some("w") == iter.next());
    assert!(Some("o") == iter.next());
    assert!(Some("r") == iter.next());
    assert!(Some("l") == iter.next());
    assert!(Some("d") == iter.next());
    assert!(Some("!") == iter.next());
    assert!(None == iter.next());
}


#[test]
fn remove_text_4() {
    let mut buf = Buffer::new();
    
    buf.insert_text("Hi\nthere\npeople\nof\nthe\nworld!", 0);
    assert!(buf.len() == 29);
    assert!(buf.root.line_count == 6);
    
    buf.remove_text(23, 29);
    
    let mut iter = buf.grapheme_iter();
    
    assert!(buf.len() == 23);
    assert!(buf.root.line_count == 6);
    assert!(Some("H") == iter.next());
    assert!(Some("i") == iter.next());
    assert!(Some("\n") == iter.next());
    assert!(Some("t") == iter.next());
    assert!(Some("h") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some("r") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some("\n") == iter.next());
    assert!(Some("p") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some("o") == iter.next());
    assert!(Some("p") == iter.next());
    assert!(Some("l") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some("\n") == iter.next());
    assert!(Some("o") == iter.next());
    assert!(Some("f") == iter.next());
    assert!(Some("\n") == iter.next());
    assert!(Some("t") == iter.next());
    assert!(Some("h") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some("\n") == iter.next());
    assert!(None == iter.next());
}


#[test]
fn remove_text_5() {
    let mut buf = Buffer::new();
    
    buf.insert_text("Hi\nthere\npeople\nof\nthe\nworld!", 0);
    assert!(buf.len() == 29);
    assert!(buf.root.line_count == 6);
    
    buf.remove_text(17, 29);
    
    let mut iter = buf.grapheme_iter();
    
    assert!(buf.len() == 17);
    assert!(buf.root.line_count == 4);
    assert!(Some("H") == iter.next());
    assert!(Some("i") == iter.next());
    assert!(Some("\n") == iter.next());
    assert!(Some("t") == iter.next());
    assert!(Some("h") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some("r") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some("\n") == iter.next());
    assert!(Some("p") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some("o") == iter.next());
    assert!(Some("p") == iter.next());
    assert!(Some("l") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some("\n") == iter.next());
    assert!(Some("o") == iter.next());
    assert!(None == iter.next());
}


#[test]
fn remove_text_6() {
    let mut buf = Buffer::new();
    
    buf.insert_text("Hello\nworld!", 0);
    assert!(buf.len() == 12);
    assert!(buf.root.line_count == 2);
    
    buf.remove_text(3, 12);
    
    let mut iter = buf.grapheme_iter();
    
    assert!(buf.len() == 3);
    assert!(buf.root.line_count == 1);
    assert!(Some("H") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some("l") == iter.next());
    assert!(None == iter.next());
}


#[test]
fn remove_text_7() {
    let mut buf = Buffer::new();
    
    buf.insert_text("Hi\nthere\nworld!", 0);
    assert!(buf.len() == 15);
    assert!(buf.root.line_count == 3);
    
    buf.remove_text(5, 15);
    
    let mut iter = buf.grapheme_iter();
    
    assert!(buf.len() == 5);
    assert!(buf.root.line_count == 2);
    assert!(Some("H") == iter.next());
    assert!(Some("i") == iter.next());
    assert!(Some("\n") == iter.next());
    assert!(Some("t") == iter.next());
    assert!(Some("h") == iter.next());
    assert!(None == iter.next());
}


#[test]
fn remove_text_8() {
    let mut buf = Buffer::new();
    
    buf.insert_text("Hello\nworld!", 0);
    assert!(buf.len() == 12);
    assert!(buf.root.line_count == 2);
    
    buf.remove_text(3, 11);
    
    let mut iter = buf.grapheme_iter();
    
    assert!(buf.len() == 4);
    assert!(buf.root.line_count == 1);
    assert!(Some("H") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some("l") == iter.next());
    assert!(Some("!") == iter.next());
    assert!(None == iter.next());
}


#[test]
fn remove_text_9() {
    let mut buf = Buffer::new();
    
    buf.insert_text("Hello\nworld!", 0);
    assert!(buf.len() == 12);
    assert!(buf.root.line_count == 2);
    
    buf.remove_text(8, 12);
    
    let mut iter = buf.grapheme_iter();
    
    assert!(buf.len() == 8);
    assert!(buf.root.line_count == 2);
    assert!(Some("H") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some("l") == iter.next());
    assert!(Some("l") == iter.next());
    assert!(Some("o") == iter.next());
    assert!(Some("\n") == iter.next());
    assert!(Some("w") == iter.next());
    assert!(Some("o") == iter.next());
    assert!(None == iter.next());
}


#[test]
fn remove_text_10() {
    let mut buf = Buffer::new();
    
    buf.insert_text("12\n34\n56\n78", 0);
    assert!(buf.len() == 11);
    assert!(buf.root.line_count == 4);
    
    buf.remove_text(4, 11);
    
    let mut iter = buf.grapheme_iter();
    
    assert!(buf.len() == 4);
    assert!(buf.root.line_count == 2);
    assert!(Some("1") == iter.next());
    assert!(Some("2") == iter.next());
    assert!(Some("\n") == iter.next());
    assert!(Some("3") == iter.next());
    assert!(None == iter.next());
}


#[test]
fn remove_text_11() {
    let mut buf = Buffer::new();
    
    buf.insert_text("1234567890", 0);
    assert!(buf.len() == 10);
    assert!(buf.root.line_count == 1);
    
    buf.remove_text(9, 10);
    
    let mut iter = buf.grapheme_iter();
    
    assert!(buf.len() == 9);
    assert!(buf.root.line_count == 1);
    assert!(Some("1") == iter.next());
    assert!(Some("2") == iter.next());
    assert!(Some("3") == iter.next());
    assert!(Some("4") == iter.next());
    assert!(Some("5") == iter.next());
    assert!(Some("6") == iter.next());
    assert!(Some("7") == iter.next());
    assert!(Some("8") == iter.next());
    assert!(Some("9") == iter.next());
    assert!(None == iter.next());
}


#[test]
fn remove_lines_1() {
    let mut buf = Buffer::new();
    
    buf.insert_text("Hi\nthere\npeople\nof\nthe\nworld!", 0);
    assert!(buf.len() == 29);
    assert!(buf.root.line_count == 6);
    
    buf.remove_lines(0, 3);
    
    let mut iter = buf.grapheme_iter();
    
    assert!(buf.len() == 13);
    assert!(buf.root.line_count == 3);
    assert!(Some("o") == iter.next());
    assert!(Some("f") == iter.next());
    assert!(Some("\n") == iter.next());
    assert!(Some("t") == iter.next());
    assert!(Some("h") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some("\n") == iter.next());
    assert!(Some("w") == iter.next());
    assert!(Some("o") == iter.next());
    assert!(Some("r") == iter.next());
    assert!(Some("l") == iter.next());
    assert!(Some("d") == iter.next());
    assert!(Some("!") == iter.next());
    assert!(None == iter.next());
}


#[test]
fn remove_lines_2() {
    let mut buf = Buffer::new();
    
    buf.insert_text("Hi\nthere\npeople\nof\nthe\nworld!", 0);
    assert!(buf.len() == 29);
    assert!(buf.root.line_count == 6);
    
    buf.remove_lines(1, 4);
    
    let mut iter = buf.grapheme_iter();
    
    assert!(buf.len() == 13);
    assert!(buf.root.line_count == 3);
    assert!(Some("H") == iter.next());
    assert!(Some("i") == iter.next());
    assert!(Some("\n") == iter.next());
    assert!(Some("t") == iter.next());
    assert!(Some("h") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some("\n") == iter.next());
    assert!(Some("w") == iter.next());
    assert!(Some("o") == iter.next());
    assert!(Some("r") == iter.next());
    assert!(Some("l") == iter.next());
    assert!(Some("d") == iter.next());
    assert!(Some("!") == iter.next());
    assert!(None == iter.next());
}


#[test]
fn remove_lines_3() {
    let mut buf = Buffer::new();
    
    buf.insert_text("Hi\nthere\npeople\nof\nthe\nworld!", 0);
    assert!(buf.len() == 29);
    assert!(buf.root.line_count == 6);
    
    buf.remove_lines(3, 6);
    
    let mut iter = buf.grapheme_iter();
    
    assert!(buf.len() == 15);
    assert!(buf.root.line_count == 3);
    assert!(Some("H") == iter.next());
    assert!(Some("i") == iter.next());
    assert!(Some("\n") == iter.next());
    assert!(Some("t") == iter.next());
    assert!(Some("h") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some("r") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some("\n") == iter.next());
    assert!(Some("p") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some("o") == iter.next());
    assert!(Some("p") == iter.next());
    assert!(Some("l") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(None == iter.next());
}


#[test]
fn pos_2d_to_closest_1d_1() {
    let mut buf = Buffer::new();
    buf.insert_text("Hi\nthere\npeople\nof\nthe\nworld!", 0);
    
    let pos = buf.pos_2d_to_closest_1d((2, 3));
    
    assert!(pos == 12);
}


#[test]
fn pos_2d_to_closest_1d_2() {
    let mut buf = Buffer::new();
    buf.insert_text("Hi\nthere\npeople\nof\nthe\nworld!", 0);
    
    let pos = buf.pos_2d_to_closest_1d((2, 10));
    
    assert!(pos == 15);
}

#[test]
fn pos_2d_to_closest_1d_3() {
    let mut buf = Buffer::new();
    buf.insert_text("Hi\nthere\npeople\nof\nthe\nworld!", 0);
    
    let pos = buf.pos_2d_to_closest_1d((10, 2));
    
    assert!(pos == 29);
}


#[test]
fn pos_1d_to_closest_2d_1() {
    let mut buf = Buffer::new();
    buf.insert_text("Hi\nthere\npeople\nof\nthe\nworld!", 0);
    
    let pos = buf.pos_1d_to_closest_2d(5);
    
    assert!(pos == (1, 2));
}


#[test]
fn pos_1d_to_closest_2d_2() {
    let mut buf = Buffer::new();
    buf.insert_text("Hi\nthere\npeople\nof\nthe\nworld!", 0);
    
    let pos = buf.pos_1d_to_closest_2d(50);
    
    assert!(pos == (5, 6));
}


#[test]
fn grapheme_iter_at_index_1() {
    let mut buf = Buffer::new();
    buf.insert_text("Hi\nthere\npeople\nof\nthe\nworld!", 0);
    
    let mut iter = buf.grapheme_iter_at_index(16);
    
    assert!(Some("o") == iter.next());
    assert!(Some("f") == iter.next());
    assert!(Some("\n") == iter.next());
    assert!(Some("t") == iter.next());
    assert!(Some("h") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some("\n") == iter.next());
    assert!(Some("w") == iter.next());
    assert!(Some("o") == iter.next());
    assert!(Some("r") == iter.next());
    assert!(Some("l") == iter.next());
    assert!(Some("d") == iter.next());
    assert!(Some("!") == iter.next());
    assert!(None == iter.next());
}


#[test]
fn grapheme_iter_at_index_2() {
    let mut buf = Buffer::new();
    buf.insert_text("Hi\nthere\npeople\nof\nthe\nworld!", 0);
    
    let mut iter = buf.grapheme_iter_at_index(29);
    
    assert!(None == iter.next());
}




