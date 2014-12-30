#![allow(dead_code)]

use std;
use std::fmt;
use std::mem;
use std::cmp::{min, max};

use self::node::{BufferNode, BufferNodeData};
use self::line::{Line, LineGraphemeIter};

//mod text_block;
//mod text_node;
mod line;
mod node;




/// A text buffer
pub struct Buffer {
    root: BufferNode,
}// 


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

    
    // /// Insert 'text' at char position 'pos'.
    // pub fn insert_text(&mut self, text: &str, pos: uint) {
    //     self.root.insert_text(text, pos);
    // }

    // 
    // /// Remove the text between char positions 'pos_a' and 'pos_b'.
    // pub fn remove_text(&mut self, pos_a: uint, pos_b: uint) {
    //     self.root.remove_text(pos_a, pos_b);
    // }

    
    /// Creates an iterator at the first character
    // pub fn root_iter<'a>(&'a self) -> TextBufferIter<'a> {
    //     let mut node_stack: Vec<&'a TextNode> = Vec::new();
    //     let mut cur_node = &self.root;
    //     
    //     loop {
    //         match cur_node.data {
    //             TextNodeData::Leaf(_) => {
    //                 break;
    //             },
    //             
    //             TextNodeData::Branch(ref left, ref right) => {
    //                 node_stack.push(&(**right));
    //                 cur_node = &(**left);
    //             }
    //         }
    //     }
    //     
    //     TextBufferIter {
    //         node_stack: node_stack,
    //         cur_block: match cur_node.data {
    //             TextNodeData::Leaf(ref tb) => tb.as_str().chars(),
    //             _ => panic!("This should never happen.")
    //         }
    //     }
    // }
    // 
    // 
    // /// Creates an iterator starting at the specified character index.
    // /// If the index is past the end of the text, then the iterator will
    // /// return None on next().
    // pub fn iter_at_char<'a>(&'a self, index: uint) -> TextBufferIter<'a> {
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







/// An iterator over a text buffer's graphemes
pub struct BufferGraphemeIter<'a> {
    node_stack: Vec<&'a BufferNode>,
    cur_line: &'a Line,
}


// impl<'a> TextBufferIter<'a> {
//     // Puts the iterator on the next line
//     pub fn next_line(&mut self) -> Option<char> {
//         // TODO: more efficient implementation, taking advantage of rope
//         // structure.
//         for c in *self {
//             if c == '\n' {
//                 return Option::Some(c);
//             }
//         }
//         
//         return Option::None;
//     }
//     
//     
//     // Skips the iterator n characters ahead
//     pub fn skip_chars(&mut self, n: uint) {
//         // TODO: more efficient implementation, taking advantage of rope
//         // structure.
//         for _ in range(0, n) {
//             if let Option::None = self.next() {
//                 break;
//             }
//         }
//     }
//     
//     
//     // Skips the iterator n characters ahead, unless it hits a newline
//     // character.  If it hits a newline character, returns true, otherwise,
//     // false.
//     pub fn skip_non_newline_chars(&mut self, n: uint) -> bool {
//         // TODO: more efficient implementation, taking advantage of rope
//         // structure.
//         for _ in range(0, n) {
//             match self.next() {
//                 Option::Some(c) => {
//                     if c == '\n' {
//                         return true;
//                     }
//                 },
//                 
//                 Option::None => {
//                     break;
//                 }
//             }
//         }
//         
//         return false;
//     }
// }
// 
// 
// impl<'a> Iterator<char> for TextBufferIter<'a> {
//     fn next(&mut self) -> Option<char> {
//         if let Option::Some(c) = self.cur_block.next() {
//             return Option::Some(c);
//         }
//       
//         loop {
//             if let Option::Some(node) = self.node_stack.pop() {
//                 match node.data {
//                     TextNodeData::Leaf(ref tb) => {
//                         self.cur_block = tb.as_str().chars();
//                       
//                         if let Option::Some(c) = self.cur_block.next() {
//                             return Option::Some(c);
//                         }
//                         else {
//                             continue;
//                         }
//                     },
//                   
//                     TextNodeData::Branch(ref left, ref right) => {
//                         self.node_stack.push(&(**right));
//                         self.node_stack.push(&(**left));
//                         continue;
//                     }
//                 }
//             }
//             else {
//                 return Option::None;
//             }
//         }
//     }
// }