#![allow(dead_code)]

use std::mem;
use std::collections::DList;

use self::node::{BufferNode, BufferNodeGraphemeIter, BufferNodeLineIter};
use self::line::{Line, LineEnding};
use string_utils::{is_line_ending, grapheme_count};

pub mod line;
mod node;


//=============================================================
// Buffer
//=============================================================

/// A text buffer
pub struct Buffer {
    text: BufferNode,
    pub line_ending_type: LineEnding,
    undo_stack: DList<Operation>,
}


impl Buffer {
    pub fn new() -> Buffer {
        Buffer {
            text: BufferNode::new(),
            line_ending_type: LineEnding::LF,
            undo_stack: DList::new(),
        }
    }

    
    pub fn len(&self) -> usize {
        self.text.grapheme_count
    }

    
    pub fn line_count(&self) -> usize {
        self.text.line_count
    }
    
    
    pub fn get_grapheme<'a>(&'a self, index: usize) -> &'a str {
        if index >= self.len() {
            panic!("Buffer::get_grapheme(): index past last grapheme.");
        }
        else {
            return self.text.get_grapheme_recursive(index);
        }
    }
    
    
    pub fn get_grapheme_width(&self, index: usize, tab_width: usize) -> usize {
        if index >= self.len() {
            panic!("Buffer::get_grapheme_width(): index past last grapheme.");
        }
        else {
            return self.text.get_grapheme_width_recursive(index, tab_width);
        }
    }
    
    
    pub fn get_line<'a>(&'a self, index: usize) -> &'a Line {
        if index >= self.line_count() {
            panic!("get_line(): index out of bounds.");
        }
        
        // NOTE: this can be done non-recursively, which would be more
        // efficient.  However, it seems likely to require unsafe code
        // if done that way.
        return self.text.get_line_recursive(index);
    }
    
    
    /// Blindly appends a line to the end of the current text without
    /// doing any sanity checks.  This is primarily for efficient
    /// file loading.
    pub fn append_line_unchecked(&mut self, line: Line) {
        self.text.append_line_unchecked_recursive(line);
    }
    
    
    /// Removes the lines in line indices [line_a, line_b).
    pub fn remove_lines(&mut self, line_a: usize, line_b: usize) {
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
        else if line_a == 0 && line_b == self.text.line_count {
            let mut temp_node = BufferNode::new();
            mem::swap(&mut (self.text), &mut temp_node);
        }
        // All other cases
        else {
            self.text.remove_lines_recursive(line_a, line_b);
            self.text.set_last_line_ending_recursive();
        }
    }

    
    pub fn pos_2d_to_closest_1d(&self, pos: (usize, usize)) -> usize {
        return self.text.pos_2d_to_closest_1d_recursive(pos);
    }


    pub fn pos_vis_2d_to_closest_1d(&self, pos: (usize, usize), tab_width: usize) -> usize {
        if pos.0 >= self.line_count() {
            return self.len();
        }
        else {
            let gs = self.pos_2d_to_closest_1d((pos.0, 0));
            let h = self.get_line(pos.0).vis_pos_to_closest_grapheme_index(pos.1, tab_width);
            return gs + h;
        }
    }

    
    pub fn pos_1d_to_closest_2d(&self, pos: usize) -> (usize, usize) {
        return self.text.pos_1d_to_closest_2d_recursive(pos);
    }
    
    
    pub fn pos_1d_to_closest_vis_2d(&self, pos: usize, tab_width: usize) -> (usize, usize) {
        let (v, h) = self.text.pos_1d_to_closest_2d_recursive(pos);
        let vis_h = self.get_line(v).grapheme_index_to_closest_vis_pos(h, tab_width);
        return (v, vis_h);
    }

    
    /// Insert 'text' at grapheme position 'pos'.
    pub fn insert_text(&mut self, text: &str, pos: usize) {
        self._insert_text(text, pos);
        
        self.undo_stack.push_back(Operation::InsertText(String::from_str(text), pos));
    }
    
    fn _insert_text(&mut self, text: &str, pos: usize) {
        self.text.insert_text(text, pos);
    }

    
    /// Remove the text between grapheme positions 'pos_a' and 'pos_b'.
    pub fn remove_text(&mut self, pos_a: usize, pos_b: usize) {
        let removed_text = self.string_from_range(pos_a, pos_b);
    
        self._remove_text(pos_a, pos_b);
        
        // Push operation to the undo stack
        self.undo_stack.push_back(Operation::RemoveText(removed_text, pos_a));
    }
    
    fn _remove_text(&mut self, pos_a: usize, pos_b: usize) {
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
        else if pos_a == 0 && pos_b == self.text.grapheme_count {
            let mut temp_node = BufferNode::new();
            mem::swap(&mut (self.text), &mut temp_node);
        }
        // All other cases
        else {
            if self.text.remove_text_recursive(pos_a, pos_b, true) {
                panic!("Buffer::remove_text(): dangling left side remains.  This should never happen!");
            }
            self.text.set_last_line_ending_recursive();
        }
    }
    
    
    /// Undoes operations that were pushed to the undo stack, and returns a
    /// cursor position that the cursor should jump to, if any.
    pub fn undo(&mut self) -> Option<usize> {
        if let Some(op) = self.undo_stack.pop_back() {
            match op {
                Operation::InsertText(ref s, p) => {
                    let size = grapheme_count(s.as_slice());
                    self._remove_text(p, p+size);
                    return Some(p);
                },
                
                Operation::RemoveText(ref s, p) => {
                    let size = grapheme_count(s.as_slice());
                    self._insert_text(s.as_slice(), p);
                    return Some(p+size);
                },
            }
        }
        
        return None;
    }
    
    
    /// Creates a String from the buffer text in grapheme range [pos_a, posb).
    fn string_from_range(&self, pos_a: usize, pos_b: usize) -> String {
        // Bounds checks
        if pos_b < pos_a {
            panic!("Buffer::string_from_range(): pos_a must be less than or equal to pos_b.");
        }
        else if pos_b > self.len() {
            panic!("Buffer::string_from_range(): specified range is past end of buffer text.");
        }
        
        let mut s = String::with_capacity(pos_b - pos_a);
        
        let mut iter = self.grapheme_iter_at_index(pos_a);
        let mut i = 0;
        let i_end = pos_b - pos_a;
        
        for g in iter {
            if i == i_end {
                break;
            }
            
            s.push_str(g);
            
            i += 1;
        }
        
        return s;
    }
    
    /// Creates an iterator at the first character
    pub fn grapheme_iter<'a>(&'a self) -> BufferGraphemeIter<'a> {
        BufferGraphemeIter {
            gi: self.text.grapheme_iter()
        }
    }
    
    
    /// Creates an iterator starting at the specified grapheme index.
    /// If the index is past the end of the text, then the iterator will
    /// return None on next().
    pub fn grapheme_iter_at_index<'a>(&'a self, index: usize) -> BufferGraphemeIter<'a> {
        BufferGraphemeIter {
            gi: self.text.grapheme_iter_at_index(index)
        }
    }
    
    
    pub fn line_iter<'a>(&'a self) -> BufferLineIter<'a> {
        BufferLineIter {
            li: self.text.line_iter()
        }
    }
    
    
    pub fn line_iter_at_index<'a>(&'a self, index: usize) -> BufferLineIter<'a> {
        BufferLineIter {
            li: self.text.line_iter_at_index(index)
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
    pub fn skip_graphemes(&mut self, n: usize) -> bool {
        self.gi.skip_graphemes(n)
    }
    
    pub fn skip_non_newline_graphemes(&mut self, n: usize) -> bool {
        let mut i: usize = 0;
        
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


impl<'a> Iterator for BufferGraphemeIter<'a> {
    type Item = &'a str;
    
    fn next(&mut self) -> Option<&'a str> {
        self.gi.next()
    }
}


pub struct BufferLineIter<'a> {
    li: BufferNodeLineIter<'a>,
}


impl<'a> Iterator for BufferLineIter<'a> {
    type Item = &'a Line;

    fn next(&mut self) -> Option<&'a Line> {
        self.li.next()
    }
}




//================================================================
// Buffer undo structures
//================================================================

enum Operation {
    InsertText(String, usize),
    RemoveText(String, usize),
}




//================================================================
// TESTS
//================================================================

mod tests {
    use super::{Buffer, Operation, BufferGraphemeIter, BufferLineIter};

    #[test]
    fn insert_text() {
        let mut buf = Buffer::new();
        
        buf.insert_text("Hello 世界!", 0);
        
        let mut iter = buf.grapheme_iter();
        
        assert!(buf.len() == 9);
        assert!(buf.text.line_count == 1);
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
        assert!(buf.text.line_count == 3);
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
        assert!(buf.text.line_count == 3);
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
        assert!(buf.text.line_count == 3);
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
        assert!(buf.text.line_count == 3);
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
        assert!(buf.text.line_count == 3);
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
        assert!(buf.text.line_count == 3);
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
        assert!(buf.text.line_count == 3);
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
        assert!(buf.text.line_count == 7);
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
        assert!(buf.text.line_count == 6);
        
        buf.remove_text(0, 3);
        
        let mut iter = buf.grapheme_iter();
        
        assert!(buf.len() == 26);
        assert!(buf.text.line_count == 5);
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
        assert!(buf.text.line_count == 6);
        
        buf.remove_text(0, 12);
        
        let mut iter = buf.grapheme_iter();
        
        assert!(buf.len() == 17);
        assert!(buf.text.line_count == 4);
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
        assert!(buf.text.line_count == 6);
        
        buf.remove_text(5, 17);
        
        let mut iter = buf.grapheme_iter();
        
        assert!(buf.len() == 17);
        assert!(buf.text.line_count == 4);
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
        assert!(buf.text.line_count == 6);
        
        buf.remove_text(23, 29);
        
        let mut iter = buf.grapheme_iter();
        
        assert!(buf.len() == 23);
        assert!(buf.text.line_count == 6);
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
        assert!(buf.text.line_count == 6);
        
        buf.remove_text(17, 29);
        
        let mut iter = buf.grapheme_iter();
        
        assert!(buf.len() == 17);
        assert!(buf.text.line_count == 4);
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
        assert!(buf.text.line_count == 2);
        
        buf.remove_text(3, 12);
        
        let mut iter = buf.grapheme_iter();
        
        assert!(buf.len() == 3);
        assert!(buf.text.line_count == 1);
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
        assert!(buf.text.line_count == 3);
        
        buf.remove_text(5, 15);
        
        let mut iter = buf.grapheme_iter();
        
        assert!(buf.len() == 5);
        assert!(buf.text.line_count == 2);
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
        assert!(buf.text.line_count == 2);
        
        buf.remove_text(3, 11);
        
        let mut iter = buf.grapheme_iter();
        
        assert!(buf.len() == 4);
        assert!(buf.text.line_count == 1);
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
        assert!(buf.text.line_count == 2);
        
        buf.remove_text(8, 12);
        
        let mut iter = buf.grapheme_iter();
        
        assert!(buf.len() == 8);
        assert!(buf.text.line_count == 2);
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
        assert!(buf.text.line_count == 4);
        
        buf.remove_text(4, 11);
        
        let mut iter = buf.grapheme_iter();
        
        assert!(buf.len() == 4);
        assert!(buf.text.line_count == 2);
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
        assert!(buf.text.line_count == 1);
        
        buf.remove_text(9, 10);
        
        let mut iter = buf.grapheme_iter();
        
        assert!(buf.len() == 9);
        assert!(buf.text.line_count == 1);
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
        assert!(buf.text.line_count == 6);
        
        buf.remove_lines(0, 3);
        
        let mut iter = buf.grapheme_iter();
        
        assert!(buf.len() == 13);
        assert!(buf.text.line_count == 3);
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
        assert!(buf.text.line_count == 6);
        
        buf.remove_lines(1, 4);
        
        let mut iter = buf.grapheme_iter();
        
        assert!(buf.len() == 13);
        assert!(buf.text.line_count == 3);
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
        assert!(buf.text.line_count == 6);
        
        buf.remove_lines(3, 6);
        
        let mut iter = buf.grapheme_iter();
        
        assert!(buf.len() == 15);
        assert!(buf.text.line_count == 3);
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
    fn string_from_range_1() {
        let mut buf = Buffer::new();
        buf.insert_text("Hi\nthere\npeople\nof\nthe\nworld!", 0);
        
        let s = buf.string_from_range(1, 12);
        
        assert!(s.as_slice() == "i\nthere\npeo");
    }
    
    
    #[test]
    fn string_from_range_2() {
        let mut buf = Buffer::new();
        buf.insert_text("Hi\nthere\npeople\nof\nthe\nworld!", 0);
        
        let s = buf.string_from_range(0, 29);
        
        assert!(s.as_slice() == "Hi\nthere\npeople\nof\nthe\nworld!");
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
    
    
}

