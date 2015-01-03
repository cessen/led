use std::mem;
use std::cmp::{min, max};

use string_utils::is_line_ending;
use super::line::{Line, LineEnding, LineGraphemeIter, str_to_line_ending};

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
    
    
    pub fn get_grapheme_recursive<'a>(&'a self, index: uint) -> &'a str {
        match self.data {
            BufferNodeData::Leaf(ref line) => {
                return line.grapheme_at_index(index);
            },
            
            BufferNodeData::Branch(ref left, ref right) => {
                if index < left.grapheme_count {
                    return left.get_grapheme_recursive(index);
                }
                else {
                    return right.get_grapheme_recursive(index - left.grapheme_count);
                }
            }
        }
    }
    
    
    pub fn get_grapheme_width_recursive(&self, index: uint) -> uint {
        match self.data {
            BufferNodeData::Leaf(ref line) => {
                return line.grapheme_width_at_index(index);
            },
            
            BufferNodeData::Branch(ref left, ref right) => {
                if index < left.grapheme_count {
                    return left.get_grapheme_width_recursive(index);
                }
                else {
                    return right.get_grapheme_width_recursive(index - left.grapheme_count);
                }
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
            BufferNodeData::Leaf(ref line) => {
                if pos.0 != 0 {
                    return self.grapheme_count;
                }
                
                if pos.1 >= self.grapheme_count {
                    if line.ending != LineEnding::None {
                        return self.grapheme_count - 1;
                    }
                    else {
                        return self.grapheme_count;
                    }
                }
                else {
                    return pos.1;
                }
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
    
    
    /// Insert 'text' at grapheme position 'pos'.
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
                    self.insert_text_recursive(text.slice(b1, b2), pos + g1);
                }
                
                g1 = g2;
                b2 += grapheme.1.len();
                g2 += 1;
                
                self.insert_line_break_recursive(str_to_line_ending(grapheme.1), pos + g1);
                
                b1 = b2;
                g1 = g2;
            }
            else {
                b2 += grapheme.1.len();
                g2 += 1;
            }
        }
        
        if g1 < g2 {
            self.insert_text_recursive(text.slice(b1, b2), pos + g1);
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
    
    
    /// Removes text between grapheme positions pos_a and pos_b.
    /// Returns true if a dangling left side remains from the removal.
    /// Returns false otherwise.
    pub fn remove_text_recursive(&mut self, pos_a: uint, pos_b: uint, is_last: bool) -> bool {
        let mut temp_node = BufferNode::new();
        let mut total_side_removal = false;
        let mut dangling_line = false;
        let mut do_merge_fix = false;
        let mut merge_line_number: uint = 0;
        
        match self.data {
            BufferNodeData::Branch(ref mut left, ref mut right) => {
                // Check for complete removal of both sides, which
                // should never happen here
                if pos_a == 0 && pos_b == self.grapheme_count {
                    panic!("remove_text_recursive(): attempting to remove entirety of self, which cannot be done from inside self.");
                }
                // Complete removal of left side
                else if pos_a == 0 && pos_b >= left.grapheme_count {
                    if pos_b > left.grapheme_count {
                        let a = 0;
                        let b = pos_b - left.grapheme_count;
                        right.remove_text_recursive(a, b, is_last);
                    }
                    
                    total_side_removal = true;
                    mem::swap(&mut temp_node, &mut (**right));
                }
                // Complete removal of right side
                else if pos_a <= left.grapheme_count && pos_b == self.grapheme_count {
                    if pos_a < left.grapheme_count {
                        let a = pos_a;
                        let b = left.grapheme_count;
                        dangling_line = left.remove_text_recursive(a, b, false);
                    }
                    
                    if is_last && !dangling_line {
                        mem::swap(&mut temp_node, &mut (**right));
                    }
                    else {
                        if is_last {
                            dangling_line = false;
                        }
                        
                        total_side_removal = true;
                        mem::swap(&mut temp_node, &mut (**left));
                    }
                }
                // Partial removal of one or both sides
                else {
                    // Right side
                    if pos_b > left.grapheme_count {
                        let a = if pos_a > left.grapheme_count {pos_a - left.grapheme_count} else {0};
                        let b = pos_b - left.grapheme_count;
                        dangling_line = right.remove_text_recursive(a, b, is_last) && !is_last;
                    }
                    
                    // Left side
                    if pos_a < left.grapheme_count {
                        let a = pos_a;
                        let b = min(pos_b, left.grapheme_count);
                        do_merge_fix = left.remove_text_recursive(a, b, false);
                        merge_line_number = left.line_count - 1;
                    }
                }
            },
            
            
            BufferNodeData::Leaf(ref mut line) => {
                let mut pos_b2 = pos_b;
                if pos_b == self.grapheme_count && line.ending != LineEnding::None {
                    line.ending = LineEnding::None;
                    pos_b2 -= 1;
                }
                line.remove_text(pos_a, pos_b2);
                
                dangling_line = line.ending == LineEnding::None && !is_last;
            },
        }
        
        // Do the merge fix if necessary
        if do_merge_fix {
            self.merge_line_with_next_recursive(merge_line_number, None);
        }
        // If one of the sides was completely removed, replace self with the
        // remaining side.
        else if total_side_removal {
            mem::swap(&mut temp_node, self);
        }
        
        self.update_stats();
        self.rebalance();
        
        return dangling_line;
    }
    
    
    pub fn append_line_unchecked_recursive(&mut self, line: Line) {
        let mut other_line = Line::new();
        
        if let BufferNodeData::Branch(_, ref mut right) = self.data {
            right.append_line_unchecked_recursive(line);
        }
        else {
            if let BufferNodeData::Leaf(ref mut this_line) = self.data {
                mem::swap(this_line, &mut other_line);
            }
            
            let new_node_a = box BufferNode::new_from_line(other_line);
            let new_node_b = box BufferNode::new_from_line(line);
            self.data = BufferNodeData::Branch(new_node_a, new_node_b);
        }
        
        self.update_stats();
        self.rebalance();
    }
    
    
    /// Removes lines in line number range [line_a, line_b)
    pub fn remove_lines_recursive(&mut self, line_a: uint, line_b: uint) {
        let mut remove_left = false;
        let mut remove_right = false;
        let mut temp_node = BufferNode::new();
        
        if let BufferNodeData::Branch(ref mut left, ref mut right) = self.data {
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
    
    
    pub fn merge_line_with_next_recursive(&mut self, line_number: uint, fetched_line: Option<&Line>) {
        match fetched_line {
            None => {
                let line: Option<Line> = self.pull_out_line_recursive(line_number + 1);
                if let Some(ref l) = line {
                    self.merge_line_with_next_recursive(line_number, Some(l));
                }
            },
            
            Some(line) => {
                match self.data {
                    BufferNodeData::Branch(ref mut left, ref mut right) => {
                        if line_number < left.line_count {
                            left.merge_line_with_next_recursive(line_number, Some(line));
                        }
                        else {
                            right.merge_line_with_next_recursive(line_number - left.line_count, Some(line));
                        }
                    },
                    
                    BufferNodeData::Leaf(ref mut line2) => {
                        line2.append_text(line.as_str());
                        line2.ending = line.ending;
                    }
                }
            }
        }
        
        self.update_stats();
        self.rebalance();
    }
    
    
    /// Removes a single line out of the text and returns it.
    pub fn pull_out_line_recursive(&mut self, line_number: uint) -> Option<Line> {
        let mut pulled_line = Line::new();
        let mut temp_node = BufferNode::new();
        let mut side_removal = false;
        
        match self.data {
            BufferNodeData::Branch(ref mut left, ref mut right) => {
                if line_number < left.line_count {
                    if let BufferNodeData::Leaf(ref mut line) = left.data {
                        mem::swap(&mut pulled_line, line);
                        mem::swap(&mut temp_node, &mut (**right));
                        side_removal = true;
                    }
                    else {
                        pulled_line = left.pull_out_line_recursive(line_number).unwrap();
                    }
                }
                else if line_number < self.line_count {
                    if let BufferNodeData::Leaf(ref mut line) = right.data {
                        mem::swap(&mut pulled_line, line);
                        mem::swap(&mut temp_node, &mut (**left));
                        side_removal = true;
                    }
                    else {
                        pulled_line = right.pull_out_line_recursive(line_number - left.line_count).unwrap();
                    }
                }
                else {
                    return None;
                }
            },
            
            
            BufferNodeData::Leaf(_) => {
                panic!("pull_out_line_recursive(): inside leaf node.  This should never happen!");
            },
        }
        
        if side_removal {
            mem::swap(&mut temp_node, self);
        }
        
        self.update_stats();
        self.rebalance();
        
        return Some(pulled_line);
    }
    
    
    /// Ensures that the last line in the node tree has no
    /// ending line break.
    pub fn set_last_line_ending_recursive(&mut self) {
        match self.data {
            BufferNodeData::Branch(_, ref mut right) => {
               right.set_last_line_ending_recursive();
            },
            
            BufferNodeData::Leaf(ref mut line) => {
                line.ending = LineEnding::None;
            },
        }
        
        self.update_stats();
    }


    /// Creates an iterator at the first grapheme
    pub fn grapheme_iter<'a>(&'a self) -> BufferNodeGraphemeIter<'a> {
        let mut node_stack: Vec<&'a BufferNode> = Vec::new();
        let mut cur_node = self;
        
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
        
        BufferNodeGraphemeIter {
            node_stack: node_stack,
            cur_line: match cur_node.data {
                BufferNodeData::Leaf(ref line) => line.grapheme_iter(),
                _ => panic!("This should never happen.")
            }
        }
    }
    
    
    /// Creates an iterator at the given grapheme index
    pub fn grapheme_iter_at_index<'a>(&'a self, index: uint) -> BufferNodeGraphemeIter<'a> {
        let mut node_stack: Vec<&'a BufferNode> = Vec::new();
        let mut cur_node = self;
        let mut grapheme_i = index;
        
        loop {
            match cur_node.data {
                BufferNodeData::Leaf(_) => {
                    break;
                },
                
                BufferNodeData::Branch(ref left, ref right) => {
                    if grapheme_i < left.grapheme_count {
                        node_stack.push(&(**right));
                        cur_node = &(**left);
                    }
                    else {
                        cur_node = &(**right);
                        grapheme_i -= left.grapheme_count;
                    }
                }
            }
        }
        
        BufferNodeGraphemeIter {
            node_stack: node_stack,
            cur_line: match cur_node.data {
                BufferNodeData::Leaf(ref line) => line.grapheme_iter_at_index(grapheme_i),
                _ => panic!("This should never happen.")
            }
        }
    }
    
    
    /// Creates a line iterator starting at the first line
    pub fn line_iter<'a>(&'a self) -> BufferNodeLineIter<'a> {
        let mut node_stack: Vec<&'a BufferNode> = Vec::new();
        let mut cur_node = self;
        
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
        
        node_stack.push(cur_node);
        
        BufferNodeLineIter {
            node_stack: node_stack,
        }
    }
    
    
    /// Creates a line iterator starting at the given line index
    pub fn line_iter_at_index<'a>(&'a self, index: uint) -> BufferNodeLineIter<'a> {
        let mut node_stack: Vec<&'a BufferNode> = Vec::new();
        let mut cur_node = self;
        let mut line_i = index;
        
        loop {
            match cur_node.data {
                BufferNodeData::Leaf(_) => {
                    break;
                },
                
                BufferNodeData::Branch(ref left, ref right) => {
                    if line_i < left.line_count { 
                        node_stack.push(&(**right));
                        cur_node = &(**left);
                    }
                    else {
                        line_i -= left.line_count;
                        cur_node = &(**right);
                    }
                }
            }
        }
        
        node_stack.push(cur_node);
        
        BufferNodeLineIter {
            node_stack: node_stack,
        }
    }


}




//=============================================================
// Node iterators
//=============================================================

/// An iterator over a text buffer's graphemes
pub struct BufferNodeGraphemeIter<'a> {
    node_stack: Vec<&'a BufferNode>,
    cur_line: LineGraphemeIter<'a>,
}


impl<'a> BufferNodeGraphemeIter<'a> {
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


impl<'a> Iterator<&'a str> for BufferNodeGraphemeIter<'a> {
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




/// An iterator over a text buffer's lines
pub struct BufferNodeLineIter<'a> {
    node_stack: Vec<&'a BufferNode>,
}


impl<'a> Iterator<&'a Line> for BufferNodeLineIter<'a> {
    fn next(&mut self) -> Option<&'a Line> {
        loop {
            if let Option::Some(node) = self.node_stack.pop() {
                match node.data {
                    BufferNodeData::Leaf(ref line) => {
                        return Some(line);
                    },
                  
                    BufferNodeData::Branch(ref left, ref right) => {
                        self.node_stack.push(&(**right));
                        self.node_stack.push(&(**left));
                        continue;
                    }
                }
            }
            else {
                return None;
            }
        }
    }
    
    
}



//====================================================================
// TESTS
//====================================================================

#[test]
fn merge_line_with_next_recursive_1() {
    let mut node = BufferNode::new();
    node.insert_text("Hi\n there!", 0);
    
    assert!(node.grapheme_count == 10);
    assert!(node.line_count == 2);
    
    node.merge_line_with_next_recursive(0, None);
    
    let mut iter = node.grapheme_iter();
    
    assert!(node.grapheme_count == 9);
    assert!(node.line_count == 1);
    assert!(Some("H") == iter.next());
    assert!(Some("i") == iter.next());
    assert!(Some(" ") == iter.next());
    assert!(Some("t") == iter.next());
    assert!(Some("h") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some("r") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some("!") == iter.next());
    assert!(None == iter.next());
}


#[test]
fn merge_line_with_next_recursive_2() {
    let mut node = BufferNode::new();
    node.insert_text("Hi\n there\n people \nof the\n world!", 0);
    
    assert!(node.grapheme_count == 33);
    assert!(node.line_count == 5);
    
    node.merge_line_with_next_recursive(2, None);
    
    let mut iter = node.grapheme_iter();
    
    assert!(node.grapheme_count == 32);
    assert!(node.line_count == 4);
    assert!(Some("H") == iter.next());
    assert!(Some("i") == iter.next());
    assert!(Some("\n") == iter.next());
    assert!(Some(" ") == iter.next());
    assert!(Some("t") == iter.next());
    assert!(Some("h") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some("r") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some("\n") == iter.next());
    assert!(Some(" ") == iter.next());
    assert!(Some("p") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some("o") == iter.next());
    assert!(Some("p") == iter.next());
    assert!(Some("l") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some(" ") == iter.next());
    assert!(Some("o") == iter.next());
    assert!(Some("f") == iter.next());
    assert!(Some(" ") == iter.next());
    assert!(Some("t") == iter.next());
    assert!(Some("h") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some("\n") == iter.next());
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
fn merge_line_with_next_recursive_3() {
    let mut node = BufferNode::new();
    node.insert_text("Hi\n there\n people \nof the\n world!", 0);
    
    assert!(node.grapheme_count == 33);
    assert!(node.line_count == 5);
    
    node.merge_line_with_next_recursive(0, None);
    
    let mut iter = node.grapheme_iter();
    
    assert!(node.grapheme_count == 32);
    assert!(node.line_count == 4);
    assert!(Some("H") == iter.next());
    assert!(Some("i") == iter.next());
    assert!(Some(" ") == iter.next());
    assert!(Some("t") == iter.next());
    assert!(Some("h") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some("r") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some("\n") == iter.next());
    assert!(Some(" ") == iter.next());
    assert!(Some("p") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some("o") == iter.next());
    assert!(Some("p") == iter.next());
    assert!(Some("l") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some(" ") == iter.next());
    assert!(Some("\n") == iter.next());
    assert!(Some("o") == iter.next());
    assert!(Some("f") == iter.next());
    assert!(Some(" ") == iter.next());
    assert!(Some("t") == iter.next());
    assert!(Some("h") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some("\n") == iter.next());
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
fn pull_out_line_recursive_1() {
    let mut node = BufferNode::new();
    node.insert_text("Hi\n there\n people \nof the\n world!", 0);
    
    assert!(node.grapheme_count == 33);
    assert!(node.line_count == 5);
    
    let line = node.pull_out_line_recursive(0).unwrap();
    assert!(line.as_str() == "Hi");
    assert!(line.ending == LineEnding::LF);
    
    let mut iter = node.grapheme_iter();
    
    assert!(node.grapheme_count == 30);
    assert!(node.line_count == 4);
    assert!(Some(" ") == iter.next());
    assert!(Some("t") == iter.next());
    assert!(Some("h") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some("r") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some("\n") == iter.next());
    assert!(Some(" ") == iter.next());
    assert!(Some("p") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some("o") == iter.next());
    assert!(Some("p") == iter.next());
    assert!(Some("l") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some(" ") == iter.next());
    assert!(Some("\n") == iter.next());
    assert!(Some("o") == iter.next());
    assert!(Some("f") == iter.next());
    assert!(Some(" ") == iter.next());
    assert!(Some("t") == iter.next());
    assert!(Some("h") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some("\n") == iter.next());
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
fn pull_out_line_recursive_2() {
    let mut node = BufferNode::new();
    node.insert_text("Hi\n there\n people \nof the\n world!", 0);
    
    assert!(node.grapheme_count == 33);
    assert!(node.line_count == 5);
    
    let line = node.pull_out_line_recursive(2).unwrap();
    assert!(line.as_str() == " people ");
    assert!(line.ending == LineEnding::LF);
    
    let mut iter = node.grapheme_iter();
    
    assert!(node.grapheme_count == 24);
    assert!(node.line_count == 4);
    assert!(Some("H") == iter.next());
    assert!(Some("i") == iter.next());
    assert!(Some("\n") == iter.next());
    assert!(Some(" ") == iter.next());
    assert!(Some("t") == iter.next());
    assert!(Some("h") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some("r") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some("\n") == iter.next());
    assert!(Some("o") == iter.next());
    assert!(Some("f") == iter.next());
    assert!(Some(" ") == iter.next());
    assert!(Some("t") == iter.next());
    assert!(Some("h") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some("\n") == iter.next());
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
fn pull_out_line_recursive_3() {
    let mut node = BufferNode::new();
    node.insert_text("Hi\n there\n people \nof the\n world!", 0);
    
    assert!(node.grapheme_count == 33);
    assert!(node.line_count == 5);
    
    let line = node.pull_out_line_recursive(4).unwrap();
    assert!(line.as_str() == " world!");
    assert!(line.ending == LineEnding::None);
    
    let mut iter = node.grapheme_iter();
    
    assert!(node.grapheme_count == 26);
    assert!(node.line_count == 4);
    assert!(Some("H") == iter.next());
    assert!(Some("i") == iter.next());
    assert!(Some("\n") == iter.next());
    assert!(Some(" ") == iter.next());
    assert!(Some("t") == iter.next());
    assert!(Some("h") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some("r") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some("\n") == iter.next());
    assert!(Some(" ") == iter.next());
    assert!(Some("p") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some("o") == iter.next());
    assert!(Some("p") == iter.next());
    assert!(Some("l") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some(" ") == iter.next());
    assert!(Some("\n") == iter.next());
    assert!(Some("o") == iter.next());
    assert!(Some("f") == iter.next());
    assert!(Some(" ") == iter.next());
    assert!(Some("t") == iter.next());
    assert!(Some("h") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some("\n") == iter.next());
    assert!(None == iter.next());
}

