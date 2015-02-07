#![allow(dead_code)]

use std::mem;
use std::path::Path;
use std::old_io::fs::File;
use std::old_io::{IoResult, BufferedReader};

use self::line::Line;
use self::node::{BufferNode, BufferNodeGraphemeIter, BufferNodeLineIter};
use self::undo_stack::{UndoStack};
use self::undo_stack::Operation::*;
use string_utils::{is_line_ending, grapheme_count};

pub mod line;
mod node;
mod undo_stack;


//=============================================================
// Buffer
//=============================================================

/// A text buffer
pub struct Buffer {
    text: BufferNode,
    file_path: Option<Path>,
    undo_stack: UndoStack,
}


impl Buffer {
    pub fn new() -> Buffer {
        Buffer {
            text: BufferNode::new(),
            file_path: None,
            undo_stack: UndoStack::new(),
        }
    }
    
    
    pub fn new_from_file(path: &Path) -> IoResult<Buffer> {
        let mut f = BufferedReader::new(try!(File::open(path)));
    
        let mut buf = Buffer {
            text: BufferNode::new(),
            file_path: Some(path.clone()),
            undo_stack: UndoStack::new(),
        };
         
        let string = f.read_to_string().unwrap();
        let mut g_iter = string.as_slice().grapheme_indices(true);
        let mut done = false;
        let mut a = 0;
        let mut b = 0;
        
        while !done {
            let mut count = 0;
            loop {
                if let Some((i, g)) = g_iter.next() {
                    count += 1;
                    b = i + g.len();
                    if is_line_ending(g) {
                        break;
                    }
                }
                else {
                    done = true;
                    break;
                }
            }
            
            if a != b {
                let substr = &string[a..b];
                let line = Line::new_from_str_unchecked(substr);
                let node = BufferNode::new_from_line_with_count_unchecked(line, count);
                buf.append_leaf_node_unchecked(node);
            }
            
            a = b;
        }
    
        // Remove initial blank line
        buf.remove_lines(0, 1);
    
        return Ok(buf);
    }


    
    
    //------------------------------------------------------------------------
    // Functions for getting information about the buffer.
    //------------------------------------------------------------------------
    
    pub fn grapheme_count(&self) -> usize {
        self.text.grapheme_count
    }

    
    pub fn line_count(&self) -> usize {
        self.text.line_count
    }

    
    
    
    //------------------------------------------------------------------------
    // Editing operations
    //------------------------------------------------------------------------
    
    /// Insert 'text' at grapheme position 'pos'.
    pub fn insert_text(&mut self, text: &str, pos: usize) {
        self._insert_text(text, pos);
        
        self.undo_stack.push(InsertText(String::from_str(text), pos));
    }
    
    fn _insert_text(&mut self, text: &str, pos: usize) {
        self.text.insert_text(text, pos);
    }

    
    /// Remove the text before grapheme position 'pos' of length 'len'.
    pub fn remove_text_before(&mut self, pos: usize, len: usize) {
        if pos >= len {
            let removed_text = self.string_from_range(pos - len, pos);
        
            self._remove_text(pos - len, pos);
            
            // Push operation to the undo stack
            self.undo_stack.push(RemoveTextBefore(removed_text, pos - len));
        }
        else {
            panic!("Buffer::remove_text_before(): attempt to remove text before beginning of buffer.");
        }
    }
    
    /// Remove the text after grapheme position 'pos' of length 'len'.
    pub fn remove_text_after(&mut self, pos: usize, len: usize) {
        let removed_text = self.string_from_range(pos, pos + len);
    
        self._remove_text(pos, pos + len);
        
        // Push operation to the undo stack
        self.undo_stack.push(RemoveTextAfter(removed_text, pos));
    }
    
    fn _remove_text(&mut self, pos_a: usize, pos_b: usize) {
        // Nothing to do
        if pos_a == pos_b {
            return;
        }
        // Bounds error
        else if pos_a > pos_b {
            panic!("Buffer::_remove_text(): pos_a must be less than or equal to pos_b.");
        }
        // Bounds error
        else if pos_b > self.grapheme_count() {
            panic!("Buffer::_remove_text(): attempt to remove text past the end of buffer.");
        }
        // Complete removal of all text
        else if pos_a == 0 && pos_b == self.text.grapheme_count {
            let mut temp_node = BufferNode::new();
            mem::swap(&mut (self.text), &mut temp_node);
        }
        // All other cases
        else {
            if self.text.remove_text_recursive(pos_a, pos_b, true) {
                panic!("Buffer::_remove_text(): dangling left side remains.  This should never happen!");
            }
            self.text.set_last_line_ending_recursive();
        }
    }
    
    
    /// Moves the text in [pos_a, pos_b) to begin at index pos_to.
    ///
    /// Note that pos_to is the desired index that the text will start at
    /// _after_ the operation, not the index before the operation.  This is a
    /// subtle but important distinction.
    pub fn move_text(&mut self, pos_a: usize, pos_b: usize, pos_to: usize) {
        self._move_text(pos_a, pos_b, pos_to);
        
        // Push operation to the undo stack
        self.undo_stack.push(MoveText(pos_a, pos_b, pos_to));
    }
    
    fn _move_text(&mut self, pos_a: usize, pos_b: usize, pos_to: usize) {
        // Nothing to do
        if pos_a == pos_b || pos_a == pos_to {
            return;
        }
        // Bounds error
        else if pos_a > pos_b {
            panic!("Buffer::_move_text(): pos_a must be less than or equal to pos_b.");
        }
        // Bounds error
        else if pos_b > self.grapheme_count() {
            panic!("Buffer::_move_text(): specified text range is beyond end of buffer.");
        }
        // Bounds error
        else if pos_to > (self.grapheme_count() - (pos_b - pos_a)) {
            panic!("Buffer::_move_text(): specified text destination is beyond end of buffer.");
        }
        // Nothing to do, because entire text specified
        else if pos_a == 0 && pos_b == self.grapheme_count() {
            return;
        }
        // All other cases
        else {
            // TODO: a more efficient implementation that directly
            // manipulates the node tree.
            let s = self.string_from_range(pos_a, pos_b);
            self._remove_text(pos_a, pos_b);
            self._insert_text(&s[], pos_to);
        }
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
    
    
    /// Blindly appends a line to the end of the current text without
    /// doing any sanity checks.  This is primarily for efficient
    /// file loading.
    pub fn append_line_unchecked(&mut self, line: Line) {
        self.text.append_line_unchecked_recursive(line);
    }
    
    
    fn append_leaf_node_unchecked(&mut self, node: BufferNode) {
        self.text.append_leaf_node_unchecked_recursive(node);
    }
    
    
    
    
    
    //------------------------------------------------------------------------
    // Undo/redo functionality
    //------------------------------------------------------------------------
    
    /// Undoes operations that were pushed to the undo stack, and returns a
    /// cursor position that the cursor should jump to, if any.
    pub fn undo(&mut self) -> Option<usize> {
        if let Some(op) = self.undo_stack.prev() {
            match op {
                InsertText(ref s, p) => {
                    let size = grapheme_count(&s[]);
                    self._remove_text(p, p+size);
                    return Some(p);
                },
                
                RemoveTextBefore(ref s, p) => {
                    let size = grapheme_count(&s[]);
                    self._insert_text(&s[], p);
                    return Some(p+size);
                },
                
                RemoveTextAfter(ref s, p) => {
                    self._insert_text(&s[], p);
                    return Some(p);
                },
                
                MoveText(pa, pb, pto) => {
                    let size = pb - pa;
                    self._move_text(pto, pto + size, pa);
                    return Some(pa);
                },
                
                _ => {
                    return None;
                },
            }
        }
        
        return None;
    }
    
    
    /// Redoes the last undone operation, and returns a cursor position that
    /// the cursor should jump to, if any.
    pub fn redo(&mut self) -> Option<usize> {
        if let Some(op) = self.undo_stack.next() {
            match op {
                InsertText(ref s, p) => {
                    let size = grapheme_count(&s[]);
                    self._insert_text(&s[], p);
                    return Some(p+size);
                },
                
                RemoveTextBefore(ref s, p) | RemoveTextAfter(ref s, p) => {
                    let size = grapheme_count(&s[]);
                    self._remove_text(p, p+size);
                    return Some(p);
                },
                
                MoveText(pa, pb, pto) => {
                    self._move_text(pa, pb, pto);
                    return Some(pa);
                },
                
                _ => {
                    return None;
                },
            }
        }
        
        return None;
    }
    
    
    
    //------------------------------------------------------------------------
    // Position conversions
    //------------------------------------------------------------------------
    
    /// Converts a grapheme index into a line number and grapheme-column
    /// number.
    ///
    /// If the index is off the end of the text, returns the line and column
    /// number of the last valid text position.
    pub fn index_to_line_col(&self, pos: usize) -> (usize, usize) {
        return self.text.index_to_line_col_recursive(pos);
    }
    
    
    /// Converts a line number and grapheme-column number into a grapheme
    /// index.
    ///
    /// If the column number given is beyond the end of the line, returns the
    /// index of the line's last valid position.  If the line number given is
    /// beyond the end of the buffer, returns the index of the buffer's last
    /// valid position.
    pub fn line_col_to_index(&self, pos: (usize, usize)) -> usize {
        return self.text.line_col_to_index_recursive(pos);
    }
    
    
    //------------------------------------------------------------------------
    // Text reading functions
    //------------------------------------------------------------------------
    
    pub fn get_grapheme<'a>(&'a self, index: usize) -> &'a str {
        if index >= self.grapheme_count() {
            panic!("Buffer::get_grapheme(): index past last grapheme.");
        }
        else {
            return self.text.get_grapheme_recursive(index);
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
    
    
    /// Creates a String from the buffer text in grapheme range [pos_a, posb).
    fn string_from_range(&self, pos_a: usize, pos_b: usize) -> String {
        // Bounds checks
        if pos_b < pos_a {
            panic!("Buffer::string_from_range(): pos_a must be less than or equal to pos_b.");
        }
        else if pos_b > self.grapheme_count() {
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
    
    
    
    //------------------------------------------------------------------------
    // Iterator creators
    //------------------------------------------------------------------------
    
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
// TESTS
//================================================================

#[cfg(test)]
mod tests {
    use super::{Buffer, BufferGraphemeIter, BufferLineIter};

    #[test]
    fn insert_text() {
        let mut buf = Buffer::new();
        
        buf.insert_text("Hello 世界!", 0);
        
        let mut iter = buf.grapheme_iter();
        
        assert!(buf.grapheme_count() == 9);
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
        
        assert!(buf.grapheme_count() == 11);
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
        
        assert!(buf.grapheme_count() == 17);
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
        
        assert!(buf.grapheme_count() == 17);
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
        
        assert!(buf.grapheme_count() == 16);
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
        
        assert!(buf.grapheme_count() == 16);
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
        
        assert!(buf.grapheme_count() == 16);
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
        
        assert!(buf.grapheme_count() == 16);
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
        
        assert!(buf.grapheme_count() == 20);
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
        assert!(buf.grapheme_count() == 29);
        assert!(buf.text.line_count == 6);
        
        buf._remove_text(0, 3);
        
        let mut iter = buf.grapheme_iter();
        
        assert!(buf.grapheme_count() == 26);
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
        assert!(buf.grapheme_count() == 29);
        assert!(buf.text.line_count == 6);
        
        buf._remove_text(0, 12);
        
        let mut iter = buf.grapheme_iter();
        
        assert!(buf.grapheme_count() == 17);
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
        assert!(buf.grapheme_count() == 29);
        assert!(buf.text.line_count == 6);
        
        buf._remove_text(5, 17);
        
        let mut iter = buf.grapheme_iter();
        
        assert!(buf.grapheme_count() == 17);
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
        assert!(buf.grapheme_count() == 29);
        assert!(buf.text.line_count == 6);
        
        buf._remove_text(23, 29);
        
        let mut iter = buf.grapheme_iter();
        
        assert!(buf.grapheme_count() == 23);
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
        assert!(buf.grapheme_count() == 29);
        assert!(buf.text.line_count == 6);
        
        buf._remove_text(17, 29);
        
        let mut iter = buf.grapheme_iter();
        
        assert!(buf.grapheme_count() == 17);
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
        assert!(buf.grapheme_count() == 12);
        assert!(buf.text.line_count == 2);
        
        buf._remove_text(3, 12);
        
        let mut iter = buf.grapheme_iter();
        
        assert!(buf.grapheme_count() == 3);
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
        assert!(buf.grapheme_count() == 15);
        assert!(buf.text.line_count == 3);
        
        buf._remove_text(5, 15);
        
        let mut iter = buf.grapheme_iter();
        
        assert!(buf.grapheme_count() == 5);
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
        assert!(buf.grapheme_count() == 12);
        assert!(buf.text.line_count == 2);
        
        buf._remove_text(3, 11);
        
        let mut iter = buf.grapheme_iter();
        
        assert!(buf.grapheme_count() == 4);
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
        assert!(buf.grapheme_count() == 12);
        assert!(buf.text.line_count == 2);
        
        buf._remove_text(8, 12);
        
        let mut iter = buf.grapheme_iter();
        
        assert!(buf.grapheme_count() == 8);
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
        assert!(buf.grapheme_count() == 11);
        assert!(buf.text.line_count == 4);
        
        buf._remove_text(4, 11);
        
        let mut iter = buf.grapheme_iter();
        
        assert!(buf.grapheme_count() == 4);
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
        assert!(buf.grapheme_count() == 10);
        assert!(buf.text.line_count == 1);
        
        buf._remove_text(9, 10);
        
        let mut iter = buf.grapheme_iter();
        
        assert!(buf.grapheme_count() == 9);
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
    fn move_text_1() {
        let mut buf = Buffer::new();
        
        buf.insert_text("Hi\nthere\npeople\nof\nthe\nworld!", 0);
        
        buf.move_text(0, 3, 2);
        
        let mut iter = buf.grapheme_iter();
        
        assert!(buf.grapheme_count() == 29);
        assert!(buf.text.line_count == 6);
        assert!(Some("t") == iter.next());
        assert!(Some("h") == iter.next());
        assert!(Some("H") == iter.next());
        assert!(Some("i") == iter.next());
        assert!(Some("\n") == iter.next());
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
    fn move_text_2() {
        let mut buf = Buffer::new();
        
        buf.insert_text("Hi\nthere\npeople\nof\nthe\nworld!", 0);
        
        buf.move_text(3, 8, 6);
        
        let mut iter = buf.grapheme_iter();
        
        assert!(buf.grapheme_count() == 29);
        assert!(buf.text.line_count == 6);
        assert!(Some("H") == iter.next());
        assert!(Some("i") == iter.next());
        assert!(Some("\n") == iter.next());
        assert!(Some("\n") == iter.next());
        assert!(Some("p") == iter.next());
        assert!(Some("e") == iter.next());
        assert!(Some("t") == iter.next());
        assert!(Some("h") == iter.next());
        assert!(Some("e") == iter.next());
        assert!(Some("r") == iter.next());
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
    fn move_text_3() {
        let mut buf = Buffer::new();
        
        buf.insert_text("Hi\nthere\npeople\nof\nthe\nworld!", 0);
        
        buf.move_text(12, 17, 6);
        
        let mut iter = buf.grapheme_iter();
        
        assert!(buf.grapheme_count() == 29);
        assert!(buf.text.line_count == 6);
        assert!(Some("H") == iter.next());
        assert!(Some("i") == iter.next());
        assert!(Some("\n") == iter.next());
        assert!(Some("t") == iter.next());
        assert!(Some("h") == iter.next());
        assert!(Some("e") == iter.next());
        assert!(Some("p") == iter.next());
        assert!(Some("l") == iter.next());
        assert!(Some("e") == iter.next());
        assert!(Some("\n") == iter.next());
        assert!(Some("o") == iter.next());
        assert!(Some("r") == iter.next());
        assert!(Some("e") == iter.next());
        assert!(Some("\n") == iter.next());
        assert!(Some("p") == iter.next());
        assert!(Some("e") == iter.next());
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
    fn move_text_4() {
        let mut buf = Buffer::new();
        
        buf.insert_text("Hi\nthere\npeople\nof\nthe\nworld!", 0);
        
        buf.move_text(23, 29, 20);
        
        let mut iter = buf.grapheme_iter();
        
        assert!(buf.grapheme_count() == 29);
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
        assert!(Some("w") == iter.next());
        assert!(Some("o") == iter.next());
        assert!(Some("r") == iter.next());
        assert!(Some("l") == iter.next());
        assert!(Some("d") == iter.next());
        assert!(Some("!") == iter.next());
        assert!(Some("h") == iter.next());
        assert!(Some("e") == iter.next());
        assert!(Some("\n") == iter.next());
        assert!(None == iter.next());
    }
    
    
    #[test]
    fn move_text_5() {
        let mut buf = Buffer::new();
        
        buf.insert_text("Hi\nthere\npeople\nof\nthe\nworld!", 0);
        
        buf.move_text(0, 29, 0);
        
        let mut iter = buf.grapheme_iter();
        
        assert!(buf.grapheme_count() == 29);
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
        assert!(Some("w") == iter.next());
        assert!(Some("o") == iter.next());
        assert!(Some("r") == iter.next());
        assert!(Some("l") == iter.next());
        assert!(Some("d") == iter.next());
        assert!(Some("!") == iter.next());
        assert!(None == iter.next());
    }
    
    
    #[test]
    fn remove_lines_1() {
        let mut buf = Buffer::new();
        
        buf.insert_text("Hi\nthere\npeople\nof\nthe\nworld!", 0);
        assert!(buf.grapheme_count() == 29);
        assert!(buf.text.line_count == 6);
        
        buf.remove_lines(0, 3);
        
        let mut iter = buf.grapheme_iter();
        
        assert!(buf.grapheme_count() == 13);
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
        assert!(buf.grapheme_count() == 29);
        assert!(buf.text.line_count == 6);
        
        buf.remove_lines(1, 4);
        
        let mut iter = buf.grapheme_iter();
        
        assert!(buf.grapheme_count() == 13);
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
        assert!(buf.grapheme_count() == 29);
        assert!(buf.text.line_count == 6);
        
        buf.remove_lines(3, 6);
        
        let mut iter = buf.grapheme_iter();
        
        assert!(buf.grapheme_count() == 15);
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
    fn line_col_to_index_1() {
        let mut buf = Buffer::new();
        buf.insert_text("Hi\nthere\npeople\nof\nthe\nworld!", 0);
        
        let pos = buf.line_col_to_index((2, 3));
        
        assert!(pos == 12);
    }
    
    
    #[test]
    fn line_col_to_index_2() {
        let mut buf = Buffer::new();
        buf.insert_text("Hi\nthere\npeople\nof\nthe\nworld!", 0);
        
        let pos = buf.line_col_to_index((2, 10));
        
        assert!(pos == 15);
    }
    
    #[test]
    fn line_col_to_index_3() {
        let mut buf = Buffer::new();
        buf.insert_text("Hi\nthere\npeople\nof\nthe\nworld!", 0);
        
        let pos = buf.line_col_to_index((10, 2));
        
        assert!(pos == 29);
    }
    
    
    #[test]
    fn index_to_line_col_1() {
        let mut buf = Buffer::new();
        buf.insert_text("Hi\nthere\npeople\nof\nthe\nworld!", 0);
        
        let pos = buf.index_to_line_col(5);
        
        assert!(pos == (1, 2));
    }
    
    
    #[test]
    fn index_to_line_col_2() {
        let mut buf = Buffer::new();
        buf.insert_text("Hi\nthere\npeople\nof\nthe\nworld!", 0);
        
        let pos = buf.index_to_line_col(50);
        
        assert!(pos == (5, 6));
    }
    
    
    #[test]
    fn string_from_range_1() {
        let mut buf = Buffer::new();
        buf.insert_text("Hi\nthere\npeople\nof\nthe\nworld!", 0);
        
        let s = buf.string_from_range(1, 12);
        
        assert!(&s[] == "i\nthere\npeo");
    }
    
    
    #[test]
    fn string_from_range_2() {
        let mut buf = Buffer::new();
        buf.insert_text("Hi\nthere\npeople\nof\nthe\nworld!", 0);
        
        let s = buf.string_from_range(0, 29);
        
        assert!(&s[] == "Hi\nthere\npeople\nof\nthe\nworld!");
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

