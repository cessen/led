use std;
use std::fmt;
use std::mem;
use std::cmp::{min, max};

use super::line::{Line, LineEnding, LineGraphemeIter};

pub enum BufferNodeData {
    Leaf(Line),
    Branch(Box<BufferNode>, Box<BufferNode>),
}

pub struct BufferNode {
    pub data: BufferNodeData,
    pub tree_height: uint,
    
    pub grapheme_count: uint,
    pub line_count: uint,
}

impl BufferNode {
    pub fn new() -> BufferNode {
        BufferNode {
            data: BufferNodeData::Leaf(Line::new()),
            tree_height: 1,
            grapheme_count: 0,
            line_count: 1,
        }
    }
    
    
    pub fn new_from_line(line: Line) -> BufferNode {
        let gc = line.grapheme_count();
    
        BufferNode {
            data: BufferNodeData::Leaf(line),
            tree_height: 1,
            grapheme_count: gc,
            line_count: 1,
        }
    }
    
    
    fn update_height(&mut self) {
        match self.data {
            BufferNodeData::Leaf(_) => {
                self.tree_height = 1;
            },
            
            BufferNodeData::Branch(ref left, ref right) => {
                self.tree_height = max(left.tree_height, right.tree_height) + 1;
            }
        }
    }
    
    
    fn update_stats(&mut self) {
        self.update_height();
        
        match self.data {
            BufferNodeData::Leaf(ref line) => {
                self.grapheme_count = line.grapheme_count();
                self.line_count = 1;
            },
            
            BufferNodeData::Branch(ref left, ref right) => {
                self.grapheme_count = left.grapheme_count + right.grapheme_count;
                self.line_count = left.line_count + right.line_count;
            }
        }
    }
    
    
    /// Rotates the tree under the node left
    fn rotate_left(&mut self) {
        let mut temp = BufferNode::new();
        
        if let BufferNodeData::Branch(_, ref mut right) = self.data {
            mem::swap(&mut temp, &mut (**right));
            
            if let BufferNodeData::Branch(ref mut left, _) = temp.data {   
                mem::swap(&mut (**left), &mut (**right));
            }
            else {
                panic!("rotate_left(): attempting to rotate node without branching right child.");
            }
        }
        else {
            panic!("rotate_left(): attempting to rotate leaf node.");
        }
        
        if let BufferNodeData::Branch(ref mut left, _) = temp.data {
            mem::swap(&mut (**left), self);
            left.update_stats();
        }
        
        mem::swap(&mut temp, self);
        self.update_stats();
    }
    
    
    /// Rotates the tree under the node right
    fn rotate_right(&mut self) {
        let mut temp = BufferNode::new();
        
        if let BufferNodeData::Branch(ref mut left, _) = self.data {
            mem::swap(&mut temp, &mut (**left));
            
            if let BufferNodeData::Branch(_, ref mut right) = temp.data {   
                mem::swap(&mut (**right), &mut (**left));
            }
            else {
                panic!("rotate_right(): attempting to rotate node without branching left child.");
            }
        }
        else {
            panic!("rotate_right(): attempting to rotate leaf node.");
        }
        
        if let BufferNodeData::Branch(_, ref mut right) = temp.data {
            mem::swap(&mut (**right), self);
            right.update_stats();
        }
        
        mem::swap(&mut temp, self);
        self.update_stats();
    }
    
    
    /// Rebalances the tree under the node
    fn rebalance(&mut self) {
        loop {
            let mut rot: int;
            
            if let BufferNodeData::Branch(ref mut left, ref mut right) = self.data {
                let height_diff = (left.tree_height as int) - (right.tree_height as int);

                // Left side higher than right side
                if height_diff > 1 {
                    let mut child_rot = false;
                    if let BufferNodeData::Branch(ref lc, ref rc) = left.data {
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
                    if let BufferNodeData::Branch(ref lc, ref rc) = right.data {
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
                // Leaf node, stop
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
    
    
    pub fn get_line_recursive<'a>(&'a self, index: uint) -> &'a Line {
        match self.data {
            BufferNodeData::Leaf(ref line) => {
                if index != 0 {
                    panic!("get_line_recursive(): at leaf, but index is not zero.  This should never happen!");
                }
                return line;
            },
            
            BufferNodeData::Branch(ref left, ref right) => {
                if index < left.line_count {
                    return left.get_line_recursive(index);
                }
                else {
                    return right.get_line_recursive(index - left.line_count);
                }
            }
        }
    }
    
    
    pub fn pos_2d_to_closest_1d_recursive(&self, pos: (uint, uint)) -> uint {
        match self.data {
            BufferNodeData::Leaf(_) => {
                if pos.0 != 0 {
                    panic!("pos_2d_to_closest_1d_recursive(): at leaf, but index is not zero.  This should never happen!");
                }
                return min(pos.1, self.grapheme_count);
            },
            
            BufferNodeData::Branch(ref left, ref right) => {
                if pos.0 < left.line_count {
                    return left.pos_2d_to_closest_1d_recursive(pos);
                }
                else {
                    return left.grapheme_count + right.pos_2d_to_closest_1d_recursive((pos.0 - left.line_count, pos.1));
                }
            }
        }
    }
    
    
    pub fn pos_1d_to_closest_2d_recursive(&self, pos: uint) -> (uint, uint) {
        match self.data {
            BufferNodeData::Leaf(_) => {
                return (0, min(pos, self.grapheme_count));
            },
            
            BufferNodeData::Branch(ref left, ref right) => {
                if pos < left.grapheme_count {
                    return left.pos_1d_to_closest_2d_recursive(pos);
                }
                else {
                    let (v, h) = right.pos_1d_to_closest_2d_recursive((pos - left.grapheme_count));
                    return (v + left.line_count, h);
                }
            }
        }
    }
    

    /// Inserts the given text string at the given grapheme position.
    /// Note: this assumes the given text has no newline graphemes.
    pub fn insert_text_recursive(&mut self, text: &str, pos: uint) {
        match self.data {
            // Find node for text to be inserted into
            BufferNodeData::Branch(ref mut left, ref mut right) => {
                if pos < left.grapheme_count {
                    left.insert_text_recursive(text, pos);
                }
                else {
                    right.insert_text_recursive(text, pos - left.grapheme_count);
                }
                
            },
            
            // Insert the text
            BufferNodeData::Leaf(ref mut line) => {
                line.insert_text(text, pos);
            },
        }
        
        self.update_stats();
    }
    
    
    /// Inserts a line break at the given grapheme position
    pub fn insert_line_break_recursive(&mut self, ending: LineEnding, pos: uint) {
        if ending == LineEnding::None {
            return;
        }
    
        let mut old_line = Line::new();
        let mut do_split: bool;
        
        match self.data {
            // Find node for the line break to be inserted into
            BufferNodeData::Branch(ref mut left, ref mut right) => {
                if pos < left.grapheme_count {
                    left.insert_line_break_recursive(ending, pos);
                }
                else {
                    right.insert_line_break_recursive(ending, pos - left.grapheme_count);
                }
                do_split = false;
            },
            
            // We need to insert the line break, so get the data we
            // need for that (can't do it here because of borrow checker).
            BufferNodeData::Leaf(ref mut line) => {
                mem::swap(&mut old_line, line);
                do_split = true;
            },
        }
        
        if do_split {
            // Insert line break
            let new_line = old_line.split(ending, pos);
            let new_node_a = box BufferNode::new_from_line(old_line);
            let new_node_b = box BufferNode::new_from_line(new_line);
            
            self.data = BufferNodeData::Branch(new_node_a, new_node_b);
            
            self.update_stats();
        }
        else {
            self.update_stats();
            self.rebalance();
        }
    }
    
    
    pub fn remove_lines_recursive(&mut self, line_a: uint, line_b: uint) {
        let mut remove_left = false;
        let mut remove_right = false;
        let mut temp_node = BufferNode::new();
        
        if let BufferNodeData::Branch(ref mut left, ref mut right) = self.data {
            // Left node completely removed
            if line_a == 0 && line_b >= left.line_count {
                remove_left = true;
            }
            // Left node partially removed
            else if line_a < left.line_count {
                let a = line_a;
                let b = min(left.line_count, line_b);
                left.remove_lines_recursive(a, b);
            }
            
            // Right node completely removed
            if line_a <= left.line_count && line_b >= (left.line_count + right.line_count) {
                remove_right = true;
            }
            // Right node partially removed
            else if line_b > left.line_count {
                let a = if line_a > left.line_count {line_a - left.line_count} else {0};
                let b = line_b - left.line_count;
                right.remove_lines_recursive(a, b);
            }
            
            // Set up for node removal
            if remove_left && remove_right {
                panic!("remove_lines_recursive(): attempting to completely remove both left and right nodes.  This should never happen!");
            }
            else if remove_left {
                mem::swap(&mut temp_node, &mut (**right));
            }
            else if remove_right {
                mem::swap(&mut temp_node, &mut (**left));
            }
        }
        else {
            panic!("remove_lines_recursive(): processing a leaf node directly.  This should never happen!");
        }
        
        // Swap out node for non-removed node
        if remove_left || remove_right {
            mem::swap(&mut temp_node, self);
        }
        
        self.update_stats();
        self.rebalance();
    }


}
