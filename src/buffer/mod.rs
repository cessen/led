#![allow(dead_code)]

use std;
use std::fmt;
use std::mem;
use std::cmp::{min, max};

use string_utils::is_line_ending;
use self::node::{BufferNode, BufferNodeData};
use self::line::{Line, LineGraphemeIter, str_to_line_ending};

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
        }
    }

    
    pub fn pos_2d_to_closest_1d(&self, pos: (uint, uint)) -> uint {
        return self.root.pos_2d_to_closest_1d_recursive(pos);
    }

    
    pub fn pos_1d_to_closest_2d(&self, pos: uint) -> (uint, uint) {
        return self.root.pos_1d_to_closest_2d_recursive(pos);
    }

    
    /// Insert 'text' at char position 'pos'.
    pub fn insert_text(&mut self, text: &str, pos: uint) {
        // Byte indices
        let mut b1: uint = 0;
        let mut b2: uint = 0;
        
        // Grapheme indices
        let mut g1: uint = 0;
        let mut g2: uint = 0;
        
        // Iterate through graphemes
        for grapheme in text.grapheme_indices(true) {
            if is_line_ending(grapheme.1) {
                if g1 < g2 {
                    self.root.insert_text_recursive(text.slice(b1, b2), pos + g1);
                }
                
                b1 = b2;
                g1 = g2;
                b2 += grapheme.1.len();
                g2 += 1;
                
                self.root.insert_line_break_recursive(str_to_line_ending(grapheme.1), pos + g1);
                
                b1 = b2;
                g1 = g2;
            }
            else {
                b2 += grapheme.1.len();
                g2 += 1;
            }
        }
        
        if g1 < g2 {
            self.root.insert_text_recursive(text.slice(b1, b2), pos + g1);
        }
    }

    // 
    // /// Remove the text between char positions 'pos_a' and 'pos_b'.
    // pub fn remove_text(&mut self, pos_a: uint, pos_b: uint) {
    //     self.root.remove_text(pos_a, pos_b);
    // }

    
    /// Creates an iterator at the first character
    pub fn grapheme_iter<'a>(&'a self) -> BufferGraphemeIter<'a> {
        let mut node_stack: Vec<&'a BufferNode> = Vec::new();
        let mut cur_node = &self.root;
        
        loop {
            match cur_node.data {
                BufferNodeData::Leaf(_) => {
                    break;
                },
                
                BufferNodeData::Branch(ref left, ref right) => {
                    node_stack.push(&(**right));
                    cur_node = &(**left);
                }
            }
        }
        
        BufferGraphemeIter {
            node_stack: node_stack,
            cur_line: match cur_node.data {
                BufferNodeData::Leaf(ref line) => line.grapheme_iter(),
                _ => panic!("This should never happen.")
            }
        }
    }
    
    
    // /// Creates an iterator starting at the specified grapheme index.
    // /// If the index is past the end of the text, then the iterator will
    // /// return None on next().
    // pub fn grapheme_iter_at_index<'a>(&'a self, index: uint) -> BufferGraphemeIter<'a> {
    //     let mut node_stack: Vec<&'a TextNode> = Vec::new();
    //     let mut cur_node = &self.root;
    //     let mut char_i = index;
    //     
    //     loop {
    //         match cur_node.data {
    //             TextNodeData::Leaf(_) => {
    //                 let mut char_iter = match cur_node.data {
    //                     TextNodeData::Leaf(ref tb) => tb.as_str().chars(),
    //                     _ => panic!("This should never happen.")
    //                 };
    //                 
    //                 while char_i > 0 {
    //                     char_iter.next();
    //                     char_i -= 1;
    //                 }
    //             
    //                 return TextBufferIter {
    //                     node_stack: node_stack,
    //                     cur_block: char_iter,
    //                 };
    //             },
    //             
    //             TextNodeData::Branch(ref left, ref right) => {
    //                 if left.char_count > char_i {
    //                     node_stack.push(&(**right));
    //                     cur_node = &(**left);
    //                 }
    //                 else {
    //                     cur_node = &(**right);
    //                     char_i -= left.char_count;
    //                 }
    //             }
    //         }
    //     }
    // }
    

}

// impl fmt::Show for Buffer {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         self.root.fmt(f)
//     }
// }





//=============================================================
// Buffer iterators
//=============================================================

/// An iterator over a text buffer's graphemes
pub struct BufferGraphemeIter<'a> {
    node_stack: Vec<&'a BufferNode>,
    cur_line: LineGraphemeIter<'a>,
}


impl<'a> BufferGraphemeIter<'a> {
    // Puts the iterator on the next line.
    // Returns true if there was a next line,
    // false if there wasn't.
    pub fn next_line(&mut self) -> bool {
        loop {
            if let Option::Some(node) = self.node_stack.pop() {
                match node.data {
                    BufferNodeData::Leaf(ref line) => {
                        self.cur_line = line.grapheme_iter();
                        return true;
                    },
                  
                    BufferNodeData::Branch(ref left, ref right) => {
                        self.node_stack.push(&(**right));
                        self.node_stack.push(&(**left));
                        continue;
                    }
                }
            }
            else {
                return false;
            }
        }
    }
    
    
    // Skips the iterator n graphemes ahead.
    // If it runs out of graphemes before reaching the desired skip count,
    // returns false.  Otherwise returns true.
    pub fn skip_graphemes(&mut self, n: uint) -> bool {
        // TODO: more efficient implementation
        for _ in range(0, n) {
            if let Option::None = self.next() {
                return false;
            }
        }
        
        return true;
    }
    
    
}


impl<'a> Iterator<&'a str> for BufferGraphemeIter<'a> {
    fn next(&mut self) -> Option<&'a str> {
        loop {
            if let Option::Some(g) = self.cur_line.next() {
                return Option::Some(g);
            }
            
            if self.next_line() {
                continue;
            }
            else {
                return Option::None;
            }
        }
    }
    
    
}



//================================================================
// TESTS
//================================================================

#[test]
fn insert_text() {
    let mut buf = Buffer::new();
    
    buf.insert_text("Hello world!", 0);
    
    let mut iter = buf.grapheme_iter();
    
    assert!(buf.root.line_count == 1);
    assert!(Some("H") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some("l") == iter.next());
    assert!(Some("l") == iter.next());
    assert!(Some("o") == iter.next());
    assert!(Some(" ") == iter.next());
    assert!(Some("w") == iter.next());
    assert!(Some("o") == iter.next());
    assert!(Some("r") == iter.next());
    assert!(Some("l") == iter.next());
    assert!(Some("d") == iter.next());
    assert!(Some("!") == iter.next());
    assert!(None == iter.next());
}


#[test]
fn insert_text_with_newlines() {
    let mut buf = Buffer::new();
    
    buf.insert_text("Hello\n world\r\n!", 0);
    
    let mut iter = buf.grapheme_iter();
    
    assert!(buf.root.line_count == 3);
    assert!(Some("H") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some("l") == iter.next());
    assert!(Some("l") == iter.next());
    assert!(Some("o") == iter.next());
    assert!(Some("\n") == iter.next());
    assert!(Some(" ") == iter.next());
    assert!(Some("w") == iter.next());
    assert!(Some("o") == iter.next());
    assert!(Some("r") == iter.next());
    assert!(Some("l") == iter.next());
    assert!(Some("d") == iter.next());
    assert!(Some("\r\n") == iter.next());
    assert!(Some("!") == iter.next());
    assert!(None == iter.next());
}