#![allow(dead_code)]

use std::cmp::{min, max};
use std::mem;
use std::fmt;
use std;


fn newline_count(text: &str) -> uint {
    let mut count = 0;
    for c in text.chars() {
        if c == '\n' {
            count += 1;
        }
    }
    return count;
}

fn char_count(text: &str) -> uint {
    let mut count = 0;
    for _ in text.chars() {
        count += 1;
    }
    return count;
}

fn char_and_newline_count(text: &str) -> (uint, uint) {
    let mut char_count = 0;
    let mut newline_count = 0;
    
    for c in text.chars() {
        char_count += 1;
        if c == '\n' {
            newline_count += 1;
        }
    }
    
    return (char_count, newline_count);
}

fn char_pos_to_byte_pos(text: &str, pos: uint) -> uint {
    let mut i: uint = 0;
    
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


/// A block of text, contiguous in memory
pub struct TextBlock {
    pub data: Vec<u8>,
}

impl TextBlock {
    /// Create a new empty text block.
    pub fn new() -> TextBlock {
        TextBlock {
            data: Vec::<u8>::new()
        }
    }
    
    /// Create a new text block with the contents of 'text'.
    pub fn new_from_str(text: &str) -> TextBlock {
        let mut tb = TextBlock {
            data: Vec::<u8>::with_capacity(text.len())
        };
        
        for b in text.bytes() {
            tb.data.push(b);
        }
        
        return tb;
    }
    
    /// Return the length of the text block in bytes.
    pub fn len(&self) -> uint {
        self.data.len()
    }
    
    /// Insert 'text' at char position 'pos'.
    pub fn insert_text(&mut self, text: &str, pos: uint) {
        // Find insertion position in bytes
        let byte_pos = char_pos_to_byte_pos(self.as_str(), pos);

        // Grow data size		
        self.data.grow(text.len(), 0);
        
        // Move old bytes forward
        let mut from = self.data.len() - text.len();
        let mut to = self.data.len();
        while from > byte_pos {
            from -= 1;
            to -= 1;
            
            self.data[to] = self.data[from];
        }
        
        // Copy new bytes in
        let mut i = byte_pos;
        for b in text.bytes() {
            self.data[i] = b;
            i += 1
        }
    }
    
    /// Remove the text between char positions 'pos_a' and 'pos_b'.
    pub fn remove_text(&mut self, pos_a: uint, pos_b: uint) {
        // Bounds checks
        if pos_a > pos_b {
            panic!("TextBlock::remove_text(): pos_a must be less than or equal to pos_b.");
        }
        
        // Find removal positions in bytes
        let byte_pos_a = char_pos_to_byte_pos(self.as_str(), pos_a);
        let byte_pos_b = char_pos_to_byte_pos(self.as_str(), pos_b);
        
        // Move bytes to fill in the gap left by the removed bytes
        let mut from = byte_pos_b;
        let mut to = byte_pos_a;
        while from < self.data.len() {
            self.data[to] = self.data[from];
            
            from += 1;
            to += 1;
        }
        
        // Remove data from the end
        let final_size = self.data.len() + byte_pos_a - byte_pos_b;
        self.data.truncate(final_size);
    }
    
    /// Returns an immutable string slice into the text block's memory
    pub fn as_str<'a>(&'a self) -> &'a str {
        unsafe {
            mem::transmute(self.data.as_slice())
        }
    }
}

impl fmt::Show for TextBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}




/// A text rope node, using TextBlocks for its underlying text
/// storage.
// TODO: record number of graphines as well, to support utf8 properly
pub struct TextNode {
    pub data: TextNodeData,
    pub tree_height: uint,
    pub char_count: uint,
    pub newline_count: uint,
}

pub enum TextNodeData {
    Leaf(TextBlock),
    Branch(Box<TextNode>, Box<TextNode>)
}

const MIN_LEAF_SIZE: uint = 64;
const MAX_LEAF_SIZE: uint = MIN_LEAF_SIZE * 2;


impl TextNode {
    pub fn new() -> TextNode {
        TextNode {
            data: TextNodeData::Leaf(TextBlock::new()),
            tree_height: 1,
            char_count: 0,
            newline_count: 0,
        }
    }
    
    pub fn new_from_str(text: &str) -> TextNode {
        TextNode {
            data: TextNodeData::Leaf(TextBlock::new_from_str(text)),
            tree_height: 1,
            char_count: char_count(text),
            newline_count: newline_count(text),
        }
    }
    
    pub fn update_height(&mut self) {
        match self.data {
            TextNodeData::Leaf(_) => {
                self.tree_height = 1;
            },
            
            TextNodeData::Branch(ref left, ref right) => {
                self.tree_height = max(left.tree_height, right.tree_height) + 1;
            }
        }
    }
    
    pub fn rotate_left(&mut self) {
        let mut temp = TextNode::new();
        
        if let TextNodeData::Branch(_, ref mut right) = self.data {
            mem::swap(&mut temp, &mut (**right));
            
            if let TextNodeData::Branch(ref mut left, _) = temp.data {   
                mem::swap(left, right);
            }
        }
        
        if let TextNodeData::Branch(ref mut left, _) = temp.data {
            mem::swap(&mut (**left), self);
            left.update_height();
        }
        
        self.update_height();
    }
    
    pub fn rotate_right(&mut self) {
        let mut temp = TextNode::new();
        
        if let TextNodeData::Branch(ref mut left, _) = self.data {
            mem::swap(&mut temp, &mut (**left));
            
            if let TextNodeData::Branch(_, ref mut right) = temp.data {   
                mem::swap(right, left);
            }
        }
        
        if let TextNodeData::Branch(_, ref mut right) = temp.data {
            mem::swap(&mut (**right), self);
            right.update_height();
        }
        
        self.update_height();
    }
    
    pub fn rebalance(&mut self) {
        loop {
            let mut rot: int = 0;
            
            if let TextNodeData::Branch(ref mut left, ref mut right) = self.data {
                let height_diff = (left.tree_height as int) - (right.tree_height as int);

                // Left side higher than right side
                if height_diff > 1 {
                    let mut child_rot = false;
                    if let TextNodeData::Branch(ref lc, ref rc) = left.data {
                        if lc.tree_height < rc.tree_height {
                            child_rot = true;
                        }
                    }
                    
                    if child_rot {
                        if let TextNodeData::Branch(_, ref mut rc) = right.data {
                            rc.rotate_right();
                        }
                    }
                    
                    rot = 1;
                }
                // Right side higher then left side
                else if height_diff < -1 {
                    let mut child_rot = false;
                    if let TextNodeData::Branch(ref lc, ref rc) = right.data {
                        if lc.tree_height > rc.tree_height {
                            child_rot = true;
                        }
                    }
                    
                    if child_rot {
                        if let TextNodeData::Branch(ref mut lc, _) = right.data {
                            lc.rotate_right();
                        }
                    }
                    
                    rot = -1;
                }
                // Balanced, stop
                else {
                    break;
                }
            }
            else {
                break;
            }
            
            if rot == 1 {
                self.rotate_right();
            }
            else if rot == 1 {
                self.rotate_left();
            }
        }
    }

    /// Recursively splits a leaf node into roughly equal-sized children,
    /// being no larger than 'max_size'.
    pub fn split(&mut self, max_size: uint) {
        if let TextNodeData::Branch(_, _) = self.data {
            panic!("TextNode::split(): attempt to split a non-leaf node.");
        }
        
        if self.char_count > max_size {
            // Split data into two new text blocks
            let mut tn1 = box TextNode::new();
            let mut tn2 = box TextNode::new();
            if let TextNodeData::Leaf(ref mut tb) = self.data {
                let pos = tb.len() / 2;
                tn1 = box TextNode::new_from_str(tb.as_str().slice(0, pos));
                tn2 = box TextNode::new_from_str(tb.as_str().slice(pos, tb.len()));
            }
            
            tn1.split(max_size);
            tn2.split(max_size);
            
            // Swap the old and new data
            let mut new_data = TextNodeData::Branch(tn1, tn2);
            mem::swap(&mut self.data, &mut new_data);
            
        }
        
        self.rebalance();
        self.update_height();
    }
    
    /// Merges the data of a non-leaf node to make it a leaf node    
    pub fn merge(&mut self) {
        if let TextNodeData::Branch(_, _) = self.data {
            let mut s: String = String::from_str("");
            
            if let TextNodeData::Branch(ref mut left, ref mut right) = self.data {
                // Merge left and right children first, to make sure we're dealing
                // with leafs
                if let TextNodeData::Branch(_, _) = left.data { left.merge(); }
                if let TextNodeData::Branch(_, _) = right.data { right.merge(); }
                
                // Push data into a string
                if let TextNodeData::Leaf(ref tb) = left.data {
                    s.push_str(tb.as_str());
                }
                if let TextNodeData::Leaf(ref tb) = right.data {
                    s.push_str(tb.as_str());
                }
            }
        
            self.data = TextNodeData::Leaf(TextBlock::new_from_str(s.as_slice()));
            self.rebalance();
            self.update_height();
        }
    }
    
    /// Insert 'text' at position 'pos'.
    pub fn insert_text(&mut self, text: &str, pos: uint) {
        if pos > self.char_count {
            panic!("TextNode::insert_text(): attempt to insert text after end of node text.");
        }
        
        match self.data {
            TextNodeData::Leaf(_) => {
                if let TextNodeData::Leaf(ref mut tb) = self.data {
                    tb.insert_text(text, pos);
                    
                    let (cc, nlc) = char_and_newline_count(text);
                    self.char_count += cc;
                    self.newline_count += nlc;
                }
                
                if self.char_count > MAX_LEAF_SIZE {
                    self.split(MAX_LEAF_SIZE);
                }
            },
            
            TextNodeData::Branch(ref mut left, ref mut right) => {
                if pos <= left.char_count {
                    left.insert_text(text, pos);
                }
                else {
                    right.insert_text(text, pos - left.char_count);
                }
                
                self.char_count = left.char_count + right.char_count;
                self.newline_count = left.newline_count + right.newline_count;
            }
        }
        
        self.rebalance();
        self.update_height();
    }
    
    /// Remove the text between byte positions 'pos_a' and 'pos_b'.
    pub fn remove_text(&mut self, pos_a: uint, pos_b: uint) {
        // Bounds checks
        if pos_a > pos_b {
            panic!("TextNode::remove_text(): pos_a must be less than or equal to pos_b.");
        }
        if pos_b > self.char_count {
            panic!("TextNode::remove_text(): attempt to remove text after end of node text.");
        }
        
        match self.data {
            TextNodeData::Leaf(ref mut tb) => {
                tb.remove_text(pos_a, pos_b);
                
                let (cc, nlc) = char_and_newline_count(tb.as_str());
                self.char_count = cc;
                self.newline_count = nlc;
            },
            
            TextNodeData::Branch(ref mut left, ref mut right) => {
                let lbc = left.char_count;
                
                if pos_a < lbc {
                    left.remove_text(pos_a, min(pos_b, lbc));
                }
                
                if pos_b > lbc {
                    right.remove_text(pos_a - min(pos_a, lbc), pos_b - lbc);
                }
                
                self.char_count = left.char_count + right.char_count;
                self.newline_count = left.newline_count + right.newline_count;
            }
        }
        
        self.rebalance();
        self.update_height();
        
        if self.char_count < MIN_LEAF_SIZE {
            self.merge();
            self.rebalance();
            self.update_height();
        }
    }
}

impl fmt::Show for TextNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.data {
            TextNodeData::Leaf(ref tb) => {
                tb.fmt(f)
            },
            
            TextNodeData::Branch(ref left, ref right) => {
                try!(left.fmt(f));
                right.fmt(f)
            }
        }
    }
}




/// A text buffer
pub struct TextBuffer {
    pub root: TextNode
}

impl TextBuffer {
    pub fn new() -> TextBuffer {
        TextBuffer {
            root: TextNode::new()
        }
    }
    
    pub fn len(&self) -> uint {
        self.root.char_count
    }
    
    /// Insert 'text' at char position 'pos'.
    pub fn insert_text(&mut self, text: &str, pos: uint) {
        self.root.insert_text(text, pos);
    }
    
    /// Remove the text between char positions 'pos_a' and 'pos_b'.
    pub fn remove_text(&mut self, pos_a: uint, pos_b: uint) {
        self.root.remove_text(pos_a, pos_b);
    }
    
    pub fn root_iter<'a>(&'a self) -> TextBufferIter<'a> {
        let mut node_stack: Vec<&'a TextNode> = Vec::new();
        let mut cur_node = &self.root;
        
        loop {
            match cur_node.data {
                TextNodeData::Leaf(_) => {
                    break;
                },
                
                TextNodeData::Branch(ref left, ref right) => {
                    node_stack.push(&(**right));
                    cur_node = &(**left);
                }
            }
        }
        
        TextBufferIter {
            node_stack: node_stack,
            cur_block: match cur_node.data {
                TextNodeData::Leaf(ref tb) => tb.as_str().chars(),
                _ => panic!("This should never happen.")
            }
        }
    }
}

impl fmt::Show for TextBuffer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.root.fmt(f)
    }
}




/// An iterator over a text buffer
pub struct TextBufferIter<'a> {
    node_stack: Vec<&'a TextNode>,
    cur_block: std::str::Chars<'a>,
}


impl<'a> TextBufferIter<'a> {
    // Puts the iterator on the next line
    pub fn next_line(&mut self) -> Option<char> {
        // TODO: more efficient implementation, taking advantage of rope
        // structure.
        for c in *self {
            if c == '\n' {
                return Option::Some(c);
            }
        }
        
        return Option::None;
    }
    
    
    // Skips the iterator n characters ahead
    pub fn skip_chars(&mut self, n: uint) {
        // TODO: more efficient implementation, taking advantage of rope
        // structure.
        for _ in range(0, n) {
            if let Option::None = self.next() {
                break;
            }
        }
    }
}


impl<'a> Iterator<char> for TextBufferIter<'a> {
    fn next(&mut self) -> Option<char> {
        if let Option::Some(c) = self.cur_block.next() {
            return Option::Some(c);
        }
      
        loop {
            if let Option::Some(node) = self.node_stack.pop() {
                match node.data {
                    TextNodeData::Leaf(ref tb) => {
                        self.cur_block = tb.as_str().chars();
                      
                        if let Option::Some(c) = self.cur_block.next() {
                            return Option::Some(c);
                        }
                        else {
                            continue;
                        }
                    },
                  
                    TextNodeData::Branch(ref left, ref right) => {
                        self.node_stack.push(&(**right));
                        self.node_stack.push(&(**left));
                        continue;
                    }
                }
            }
            else {
                return Option::None;
            }
        }
    }
}