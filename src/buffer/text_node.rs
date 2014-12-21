#![allow(dead_code)]

use std::fmt;
use std::mem;
use std::cmp::{min, max};

use super::utils::{newline_count, char_count, char_and_newline_count};
use super::text_block::TextBlock;

const MIN_LEAF_SIZE: uint = 64;
const MAX_LEAF_SIZE: uint = MIN_LEAF_SIZE * 2;

pub enum IndexOrOffset {
    Index(uint),
    Offset(uint)
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
    
    pub fn update_stats(&mut self) {
        match self.data {
            TextNodeData::Leaf(ref tb) => {
                self.tree_height = 1;
                let (cc, nlc) = char_and_newline_count(tb.as_str());
                self.char_count = cc;
                self.newline_count = nlc;
            },
            
            TextNodeData::Branch(ref left, ref right) => {
                self.tree_height = max(left.tree_height, right.tree_height) + 1;
                self.char_count = left.char_count + right.char_count;
                self.newline_count = left.newline_count + right.newline_count;
            }
        }
    }
    
    pub fn rotate_left(&mut self) {
        let mut temp = TextNode::new();
        
        if let TextNodeData::Branch(_, ref mut right) = self.data {
            mem::swap(&mut temp, &mut (**right));
            
            if let TextNodeData::Branch(ref mut left, _) = temp.data {   
                mem::swap(&mut (**left), &mut (**right));
            }
            else {
                panic!("rotate_left(): attempting to rotate node without branching right child.");
            }
        }
        else {
            panic!("rotate_left(): attempting to rotate leaf node.");
        }
        
        if let TextNodeData::Branch(ref mut left, _) = temp.data {
            mem::swap(&mut (**left), self);
            left.update_stats();
        }
        
        mem::swap(&mut temp, self);
        self.update_stats();
    }
    
    pub fn rotate_right(&mut self) {
        let mut temp = TextNode::new();
        
        if let TextNodeData::Branch(ref mut left, _) = self.data {
            mem::swap(&mut temp, &mut (**left));
            
            if let TextNodeData::Branch(_, ref mut right) = temp.data {   
                mem::swap(&mut (**right), &mut (**left));
            }
            else {
                panic!("rotate_right(): attempting to rotate node without branching left child.");
            }
        }
        else {
            panic!("rotate_right(): attempting to rotate leaf node.");
        }
        
        if let TextNodeData::Branch(_, ref mut right) = temp.data {
            mem::swap(&mut (**right), self);
            right.update_stats();
        }
        
        mem::swap(&mut temp, self);
        self.update_stats();
    }
    
    pub fn rebalance(&mut self) {
        loop {
            let mut rot: int;
            
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
                        left.rotate_left();
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
                        right.rotate_right();
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
            else if rot == -1 {
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
    
    /// Find the closest 1d text position that represents the given
    /// 2d position well.
    pub fn pos_2d_to_closest_1d(&self, offset: uint, pos: (uint, uint)) -> IndexOrOffset {
        match self.data {
            TextNodeData::Leaf(ref tb) => {
                let mut iter = tb.as_str().chars();
                let mut i = 0;
                let mut line = 0;
                let mut col = offset;
                
                for c in iter {
                    // Check if we've hit a relevant character
                    if line > pos.0 || (line == pos.0 && col >= pos.1) {
                        break;
                    }
                    
                    // Increment counters
                    if c == '\n' {
                        line += 1;
                        col = 0;
                    }
                    else {
                        col += 1;
                    }
                    i += 1;
                }
            
                // If we've reached the end of this text block but
                // haven't reached the target position, return an
                // offset of the amount of this line already consumed.
                if pos.0 > line || (pos.0 == line && pos.1 > col) {
                    return IndexOrOffset::Offset(col);
                }
                
                // Otherwise, we've found it!
                return IndexOrOffset::Index(i);
            },
            
            TextNodeData::Branch(ref left, ref right) => {
                // Left child
                if pos.0 <= left.newline_count {
                    match left.pos_2d_to_closest_1d(offset, pos) {
                        IndexOrOffset::Index(il) => {
                            return IndexOrOffset::Index(il);
                        },
                        
                        IndexOrOffset::Offset(il) => {
                            match right.pos_2d_to_closest_1d(il, (pos.0 - left.newline_count, pos.1)) {
                                IndexOrOffset::Index(ir) => {
                                    return IndexOrOffset::Index(ir + left.char_count);
                                },
                                
                                IndexOrOffset::Offset(ir) => {
                                    return IndexOrOffset::Offset(ir);
                                }
                            }
                        }
                    }
                }
                // Right child
                else {
                    match right.pos_2d_to_closest_1d(0, (pos.0 - left.newline_count, pos.1)) {
                        IndexOrOffset::Index(ir) => {
                            return IndexOrOffset::Index(ir);
                        },
                        
                        IndexOrOffset::Offset(ir) => {
                            return IndexOrOffset::Offset(ir);
                        }
                    }
                }
            }
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