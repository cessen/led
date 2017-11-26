#![allow(dead_code)]
//#![feature(test)]
//#![feature(unicode)]

//extern crate test;
extern crate unicode_segmentation;

mod string_utils;
mod tests;
mod benches;

use std::cmp::{min, max};
use std::mem;
use std::str::Chars;
use unicode_segmentation::{UnicodeSegmentation, Graphemes};
use string_utils::{
    char_count,
    char_grapheme_line_ending_count,
    grapheme_count_is_less_than,
    graphemes_are_mergeable,
    char_pos_to_byte_pos,
    char_pos_to_grapheme_pos,
    grapheme_pos_to_char_pos,
    insert_text_at_char_index,
    remove_text_between_char_indices,
    split_string_at_char_index,
    split_string_at_grapheme_index,
    is_line_ending,
};


pub const MIN_NODE_SIZE: usize = 64;
pub const MAX_NODE_SIZE: usize = MIN_NODE_SIZE * 2;


/// A rope data structure for storing text in a format that is efficient
/// for insertion and removal even for extremely large strings.
#[derive(Debug)]
pub struct Rope {
    data: RopeData,
    char_count_: usize,
    grapheme_count_: usize,
    line_ending_count_: usize,
    tree_height: u32,
}


#[derive(Debug)]
enum RopeData {
    Leaf(String),
    Branch(Box<Rope>, Box<Rope>),
}


impl Rope {
    /// Creates a new empty rope
    pub fn new() -> Rope {
        Rope {
            data: RopeData::Leaf(String::new()),
            char_count_: 0,
            grapheme_count_: 0,
            line_ending_count_: 0,
            tree_height: 1,
        }
    }
    

    /// Creates a new rope from a string slice    
    pub fn from_str(s: &str) -> Rope {
        let mut rope_stack: Vec<Rope> = Vec::new();
        
        let mut s1 = s;
        loop {
            // Get the next chunk of the string to add
            let mut byte_i = 0;
            let mut le_count = 0;
            let mut c_count = 0;
            let mut g_count = 0;
            for (bi, g) in UnicodeSegmentation::grapheme_indices(s1, true) {
                byte_i = bi + g.len();
                g_count += 1;
                c_count += char_count(g);
                if is_line_ending(g) {
                    le_count += 1;
                }
                if g_count >= MAX_NODE_SIZE {
                    break;
                }
            }
            if g_count == 0 {
                break;
            }
            let chunk = &s1[..byte_i];
            
            // Add chunk
            rope_stack.push(Rope {
                data: RopeData::Leaf(chunk.to_string()),
                char_count_: c_count,
                grapheme_count_: g_count,
                line_ending_count_: le_count,
                tree_height: 1,
            });
            
            // Do merges
            loop {
                let rsl = rope_stack.len();
                if rsl > 1 && rope_stack[rsl-2].tree_height <= rope_stack[rsl-1].tree_height {
                    let right = Box::new(rope_stack.pop().unwrap());
                    let left = Box::new(rope_stack.pop().unwrap());
                    let h = max(left.tree_height, right.tree_height) + 1;
                    let lc = left.line_ending_count_ + right.line_ending_count_;
                    let gc = left.grapheme_count_ + right.grapheme_count_;
                    let cc = left.char_count_ + right.char_count_;
                    rope_stack.push(Rope {
                        data: RopeData::Branch(left, right),
                        char_count_: cc,
                        grapheme_count_: gc,
                        line_ending_count_: lc,
                        tree_height: h,
                    });
                }
                else {
                    break;
                }
            }
            
            s1 = &s1[byte_i..];
        }
        
        
        // Handle possible final unmerged case
        let rope = if rope_stack.len() == 0 {
            Rope::new()
        }
        else {
            while rope_stack.len() > 1 {
                let right = rope_stack.pop().unwrap();
                let mut left = rope_stack.pop().unwrap();
                left.append_right(right);
                rope_stack.push(left);
            }
            rope_stack.pop().unwrap()
        };
        
        return rope;
    }
    
    /// Creates a new rope from a string, consuming the string
    pub fn from_string(s: String) -> Rope {
        // TODO: special case short strings?
        Rope::from_str(&s[..])
    }
    
    pub fn char_count(&self) -> usize {
        return self.char_count_;
    }
    
    pub fn grapheme_count(&self) -> usize {
        return self.grapheme_count_;
    }
    
    pub fn line_ending_count(&self) -> usize {
        return self.line_ending_count_;
    }
    
    
    /// Returns the number of graphemes between char indices pos_a and pos_b.
    /// This is not as simple as a subtraction of char_index_to_grapheme_index()
    /// calls, because the char indices may split graphemes.
    /// Runs in O(log N) time.
    pub fn grapheme_count_in_char_range(&self, pos_a: usize, pos_b: usize) -> usize {
        assert!(pos_a <= pos_b, "Rope::grapheme_count_in_char_range(): pos_a must be less than or equal to pos_b.");
        assert!(pos_b <= self.char_count(), "Rope::grapheme_count_in_char_range(): attempted to get grapheme count beyond the end of the text.");
        
        let ga = self.char_index_to_grapheme_index(pos_a);
        let gb = self.char_index_to_grapheme_index(pos_b);
        let cb = self.grapheme_index_to_char_index(gb);
        
        if pos_b == cb {
            return gb - ga;
        }
        else {
            return 1 + gb - ga;
        }
    }
    
    
    /// Returns the index of the grapheme that the given char index is a
    /// part of.
    pub fn char_index_to_grapheme_index(&self, pos: usize) -> usize {
        assert!(pos <= self.char_count(), "Rope::char_index_to_grapheme_index(): attempted to index beyond the end of the text.");
        
        match self.data {
            RopeData::Leaf(ref text) => {
                return char_pos_to_grapheme_pos(text, pos);
            },
            
            RopeData::Branch(ref left, ref right) => {
                if pos < left.char_count_ {
                    return left.char_index_to_grapheme_index(pos);
                }
                else {
                    return left.grapheme_count_ + right.char_index_to_grapheme_index(pos - left.char_count_);
                }
            },
        }
        
        unreachable!()
    }
    
    
    /// Returns the beginning char index of the given grapheme index.
    pub fn grapheme_index_to_char_index(&self, pos: usize) -> usize {
        assert!(pos <= self.grapheme_count(), "Rope::grapheme_index_to_char_index(): attempted to index beyond the end of the text.");
        
        match self.data {
            RopeData::Leaf(ref text) => {
                return grapheme_pos_to_char_pos(text, pos);
            },
            
            RopeData::Branch(ref left, ref right) => {
                if pos < left.grapheme_count_ {
                    return left.grapheme_index_to_char_index(pos);
                }
                else {
                    return left.char_count_ + right.grapheme_index_to_char_index(pos - left.grapheme_count_);
                }
            },
        }
        
        unreachable!()
    }
    
    
    /// Returns the index of the line that the given char index is on.
    pub fn char_index_to_line_index(&self, pos: usize) -> usize {
        assert!(pos <= self.char_count(), "Rope::char_index_to_line_index(): attempted to index beyond the end of the text.");
    
        match self.data {
            RopeData::Leaf(ref text) => {
                let mut ci = 0;
                let mut lei = 0;
                for g in UnicodeSegmentation::graphemes(&text[..], true) {
                    if ci == pos {
                        break;
                    }
                    ci += char_count(g);
                    if ci > pos {
                        break;
                    }
                    if is_line_ending(g) {
                        lei += 1;
                    }
                }
                return lei;
            },
            
            RopeData::Branch(ref left, ref right) => {
                if pos < left.char_count_ {
                    return left.char_index_to_line_index(pos);
                }
                else {
                    return right.char_index_to_line_index(pos - left.char_count_) + left.line_ending_count_;
                }
            },
        }
    }
    
    
    /// Returns the char index at the start of the given line index.
    pub fn line_index_to_char_index(&self, li: usize) -> usize {
        assert!(li <= self.line_ending_count(), "Rope::line_index_to_char_index(): attempted to index beyond the end of the text.");
        
        // Special case for the beginning of the rope
        if li == 0 {
            return 0;
        }
        
        // General cases
        match self.data {
            RopeData::Leaf(ref text) => {
                let mut ci = 0;
                let mut lei = 0;
                for g in UnicodeSegmentation::graphemes(&text[..], true) {
                    ci += char_count(g);
                    if is_line_ending(g) {
                        lei += 1;
                    }
                    if lei == li {
                        break;
                    }
                }
                return ci;
            },
            
            RopeData::Branch(ref left, ref right) => {
                if li <= left.line_ending_count_ {
                    return left.line_index_to_char_index(li);
                }
                else {
                    return right.line_index_to_char_index(li - left.line_ending_count_) + left.char_count_;
                }
            },
        }
    }
    
    
    pub fn char_at_index(&self, index: usize) -> char {
        assert!(index < self.char_count(), "Rope::char_at_index(): attempted to fetch char that is outside the bounds of the text.");
        
        match self.data {
            RopeData::Leaf(ref text) => {
                let mut i: usize = 0;
                for c in text.chars() {
                    if i == index {
                        return c;
                    }
                    i += 1;
                }
                unreachable!();
            },
            
            RopeData::Branch(ref left, ref right) => {
                if index < left.char_count() {
                    return left.char_at_index(index);
                }
                else {
                    return right.char_at_index(index - left.char_count());
                }
            },
        }
    }
    
    
    pub fn grapheme_at_index<'a>(&'a self, index: usize) -> &'a str {
        assert!(index < self.grapheme_count(), "Rope::grapheme_at_index(): attempted to fetch grapheme that is outside the bounds of the text.");
        
        match self.data {
            RopeData::Leaf(ref text) => {
                let mut i: usize = 0;
                for g in UnicodeSegmentation::graphemes(&text[..], true) {
                    if i == index {
                        return g;
                    }
                    i += 1;
                }
                unreachable!();
            },
            
            RopeData::Branch(ref left, ref right) => {
                if index < left.grapheme_count() {
                    return left.grapheme_at_index(index);
                }
                else {
                    return right.grapheme_at_index(index - left.grapheme_count());
                }
            },
        }
    }
    
    
    /// Inserts the given text at the given char index.
    /// For small lengths of 'text' runs in O(log N) time.
    /// For large lengths of 'text', dunno.  But it seems to perform
    /// sub-linearly, at least.
    pub fn insert_text_at_char_index(&mut self, text: &str, pos: usize) {
        assert!(pos <= self.char_count(), "Rope::insert_text_at_char_index(): attempted to insert text at a position beyond the end of the text.");
    
        // Insert text    
        let cc = self.char_count_;
        self.insert_text_at_char_index_without_seam_check(text, pos);
        let cc2 = self.char_count_;
        
        // Repair possible grapheme seams
        self.repair_grapheme_seam(pos);
        self.repair_grapheme_seam(pos + cc2 - cc);
    }
    
    
    /// Removes the text between the given char indices.
    /// For small distances between pos_a and pos_b runs in O(log N) time.
    /// For large distances, dunno.  If it becomes a performance bottleneck,
    /// can special-case that to two splits and an append, which are all
    /// sublinear.
    pub fn remove_text_between_char_indices(&mut self, pos_a: usize, pos_b: usize) {
        assert!(pos_a <= pos_b, "Rope::remove_text_between_char_indices(): pos_a must be less than or equal to pos_b.");
        assert!(pos_b <= self.char_count(), "Rope::remove_text_between_char_indices(): attempted to remove text beyond the end of the text.");
        
        self.remove_text_between_char_indices_without_seam_check(pos_a, pos_b);
        self.repair_grapheme_seam(pos_a);
    }
    
    
    /// Splits a rope into two pieces from the given char index.
    /// The first piece remains in this rope, the second piece is returned
    /// as a new rope.
    /// I _think_ this runs in O(log N) time, but this needs more analysis to
    /// be sure.  It is at least sublinear.
    pub fn split_at_char_index(&mut self, pos: usize) -> Rope {
        assert!(pos <= self.char_count(), "Rope::split_at_char_index(): attempted to split text at a position beyond the end of the text.");
    
        let mut left = Rope::new();
        let mut right = Rope::new();
        
        self.split_recursive(pos, &mut left, &mut right);
        
        mem::swap(self, &mut left);
        return right;
    }
    

    /// Appends another rope to the end of this one, consuming the other rope.
    /// Runs in O(log N) time.
    pub fn append(&mut self, rope: Rope) {
        let cc = self.char_count_;
        self.append_without_seam_check(rope);
        self.repair_grapheme_seam(cc);
    }    
    
    
    /// Makes a copy of the rope as a string.
    /// Runs in O(N) time.
    pub fn to_string(&self) -> String {
        let mut s = String::new();

        for chunk in self.chunk_iter() {
            s.push_str(chunk);
        }
        
        return s;
    }
    
    
    /// Creates a chunk iterator for the rope
    pub fn chunk_iter<'a>(&'a self) -> RopeChunkIter<'a> {
        self.chunk_iter_at_char_index(0).1
    }
    
    
    /// Creates a chunk iter starting at the chunk containing the given
    /// char index.  Returns the chunk iter and its starting char index.
    pub fn chunk_iter_at_char_index<'a>(&'a self, index: usize) -> (usize, RopeChunkIter<'a>) {
        assert!(index <= self.char_count(), "Rope::chunk_iter_at_char_index(): attempted to create an iterator starting beyond the end of the text.");
        
        let mut node_stack: Vec<&'a Rope> = Vec::new();
        let mut cur_node = self;
        let mut char_i = index;
        
        // Find the right rope node, and populate the stack at the same time
        loop {
            match cur_node.data {
                RopeData::Leaf(_) => {
                    node_stack.push(cur_node);
                    break;
                },
                
                RopeData::Branch(ref left, ref right) => {
                    if char_i < left.char_count_ {
                        node_stack.push(&(**right));
                        cur_node = &(**left);
                    }
                    else {
                        cur_node = &(**right);
                        char_i -= left.char_count_;
                    }
                }
            }
        }
        
        (index - char_i, RopeChunkIter {node_stack: node_stack})
    }
    
    
    /// Creates an iterator at the first char of the rope
    pub fn char_iter<'a>(&'a self) -> RopeCharIter<'a> {
        self.char_iter_at_index(0)
    }
    
    
    /// Creates an iterator starting at the given char index
    pub fn char_iter_at_index<'a>(&'a self, index: usize) -> RopeCharIter<'a> {
        assert!(index <= self.char_count(), "Rope::char_iter_at_index(): attempted to create an iterator starting beyond the end of the text.");
        
        let (char_i, mut chunk_iter) = self.chunk_iter_at_char_index(index);
        
        // Create the char iter for the current node
        let mut citer = if let Some(text) = chunk_iter.next() {
            (&text[..]).chars()
        }
        else {
            unreachable!()
        };
        
        // Get to the right spot in the iter
        for _ in char_i..index {
            citer.next();
        }
        
        // Create the rope grapheme iter
        return RopeCharIter {
            chunk_iter: chunk_iter,
            cur_chunk: citer,
            length: None,
        };
    }
    
    
    /// Creates an iterator that starts at pos_a and stops just before pos_b.
    pub fn char_iter_between_indices<'a>(&'a self, pos_a: usize, pos_b: usize) -> RopeCharIter<'a> {
        assert!(pos_a <= pos_b, "Rope::char_iter_between_indices(): pos_a must be less than or equal to pos_b.");
        assert!(pos_b <= self.char_count(), "Rope::char_iter_between_indices(): attempted to create an iterator starting beyond the end of the text.");
    
        let mut iter = self.char_iter_at_index(pos_a);
        iter.length = Some(pos_b - pos_a);
        return iter;
    }
    
    
    /// Creates an iterator at the first grapheme of the rope
    pub fn grapheme_iter<'a>(&'a self) -> RopeGraphemeIter<'a> {
        self.grapheme_iter_at_index(0)
    }
    
    
    /// Creates an iterator at the given grapheme index
    pub fn grapheme_iter_at_index<'a>(&'a self, index: usize) -> RopeGraphemeIter<'a> {
        assert!(index <= self.grapheme_count(), "Rope::grapheme_iter_at_index(): attempted to create an iterator starting beyond the end of the text.");
        
        let cindex = self.grapheme_index_to_char_index(index);
        return self.grapheme_iter_at_char_index(cindex);
    }
    
    
    /// Creates an iterator that starts a pos_a and stops just before pos_b.
    pub fn grapheme_iter_between_indices<'a>(&'a self, pos_a: usize, pos_b: usize) -> RopeGraphemeIter<'a> {
        assert!(pos_a <= pos_b, "Rope::grapheme_iter_between_indices(): pos_a must be less than or equal to pos_b.");
        assert!(pos_b <= self.grapheme_count(), "Rope::grapheme_iter_between_indices(): attempted to create an iterator starting beyond the end of the text.");
    
        let mut iter = self.grapheme_iter_at_index(pos_a);
        let cpos_a = self.grapheme_index_to_char_index(pos_a);
        let cpos_b = self.grapheme_index_to_char_index(pos_b);
        iter.length = Some(cpos_b - cpos_a);
        return iter;
    }
    
    
    /// Creates an iterator over the lines in the rope.
    pub fn line_iter<'a>(&'a self) -> RopeLineIter<'a> {
        RopeLineIter {
            rope: self,
            li: 0,
        }
    }
    
    
    /// Creates an iterator over the lines in the rope, starting at the given
    /// line index.
    pub fn line_iter_at_index<'a>(&'a self, index: usize) -> RopeLineIter<'a> {
        assert!(index <= (self.line_ending_count()+1), "Rope::line_iter_at_index(): attempted to create an iterator starting beyond the end of the text.");
        
        RopeLineIter {
            rope: self,
            li: index,
        }
    }
    
    
    // Creates a slice into the Rope, between char indices pos_a and pos_b.
    pub fn slice<'a>(&'a self, pos_a: usize, pos_b: usize) -> RopeSlice<'a> {
        assert!(pos_a <= pos_b, "Rope::slice(): pos_a must be less than or equal to pos_b.");
        assert!(pos_b <= self.char_count(), "Rope::slice(): attempted to create a slice extending beyond the end of the text.");
        
        let a = pos_a;
        let b = min(self.char_count_, pos_b);
        
        RopeSlice {
            rope: self,
            start: a,
            end: b,
        }
    }
    
    
    // Creates a graphviz document of the Rope's structure, and returns
    // it as a string.  For debugging purposes.
    pub fn to_graphviz(&self) -> String {
        let mut text = "digraph {\n".to_string();
        self.to_graphviz_recursive(&mut text, "s".to_string());
        text.push_str("}\n");
        return text;
    }
    
    
    //================================================================
    // Private utility functions
    //================================================================
    
    
    fn to_graphviz_recursive(&self, text: &mut String, name: String) {
        match self.data {
            RopeData::Leaf(_) => {
                text.push_str(&(format!("{} [label=\"cc={}\\ngc={}\\nlec={}\"];\n", name, self.char_count_, self.grapheme_count_, self.line_ending_count_))[..]);
            },
            
            RopeData::Branch(ref left, ref right) => {
                let mut lname = name.clone();
                let mut rname = name.clone();
                lname.push('l');
                rname.push('r');
                text.push_str(&(format!("{} [shape=box, label=\"h={}\\ncc={}\\ngc={}\\nlec={}\"];\n", name, self.tree_height, self.char_count_, self.grapheme_count_, self.line_ending_count_))[..]);
                text.push_str(&(format!("{} -> {{ {} {} }};\n", name, lname, rname))[..]);
                left.to_graphviz_recursive(text, lname);
                right.to_graphviz_recursive(text, rname);
            }
        }
    }
    
    
    fn is_leaf(&self) -> bool {
        if let RopeData::Leaf(_) = self.data {
            true
        }
        else {
            false
        }
    }
    

    /// Non-recursively updates the stats of a node    
    fn update_stats(&mut self) {
        match self.data {
            RopeData::Leaf(ref text) => {
                let (cc, gc, lec) = char_grapheme_line_ending_count(text);
                self.char_count_ = cc;
                self.grapheme_count_ = gc;
                self.line_ending_count_ = lec;
                self.tree_height = 1;
            },
            
            RopeData::Branch(ref left, ref right) => {
                self.char_count_ = left.char_count_ + right.char_count_;
                self.grapheme_count_ = left.grapheme_count_ + right.grapheme_count_;
                self.line_ending_count_ = left.line_ending_count_ + right.line_ending_count_;
                self.tree_height = max(left.tree_height, right.tree_height) + 1;
            }
        }
    }
    
    
    fn split_recursive(&mut self, pos: usize, left: &mut Rope, right: &mut Rope) {
        match self.data {
            RopeData::Leaf(ref text) => {
                // Split the text into two new nodes
                let mut l_text = text.clone();
                let r_text = split_string_at_char_index(&mut l_text, pos);
                let new_rope_l = Rope::from_string(l_text);
                let mut new_rope_r = Rope::from_string(r_text);
                
                // Append the nodes to their respective sides
                left.append_without_seam_check(new_rope_l);
                mem::swap(right, &mut new_rope_r);
                right.append_without_seam_check(new_rope_r);
            },
            
            RopeData::Branch(ref mut left_b, ref mut right_b) => {
                let mut l = Rope::new();
                let mut r = Rope::new();
                mem::swap(&mut **left_b, &mut l);
                mem::swap(&mut **right_b, &mut r);
                
                // Split is on left side
                if pos < l.char_count_ {
                    // Append the right split to the right side
                    mem::swap(right, &mut r);
                    right.append_without_seam_check(r);
                    
                    // Recurse
                    if let RopeData::Branch(_, ref mut new_left) = left.data {
                        if let RopeData::Branch(ref mut new_right, _) = right.data {
                            l.split_recursive(pos, new_left, new_right);
                        }
                        else {
                            l.split_recursive(pos, new_left, right);
                        }
                    }
                    else {
                        if let RopeData::Branch(ref mut new_right, _) = right.data {
                            l.split_recursive(pos, left, new_right);
                        }
                        else {
                            l.split_recursive(pos, left, right);
                        }
                    }
                }
                // Split is on right side
                else {
                    // Append the left split to the left side
                    let new_pos = pos - l.char_count_;
                    left.append_without_seam_check(l);
                    
                    // Recurse
                    if let RopeData::Branch(_, ref mut new_left) = left.data {
                        if let RopeData::Branch(ref mut new_right, _) = right.data {
                            r.split_recursive(new_pos, new_left, new_right);
                        }
                        else {
                            r.split_recursive(new_pos, new_left, right);
                        }
                    }
                    else {
                        if let RopeData::Branch(ref mut new_right, _) = right.data {
                            r.split_recursive(new_pos, left, new_right);
                        }
                        else {
                            r.split_recursive(new_pos, left, right);
                        }
                    }
                }
            },
            
        }
        
        left.rebalance();
        right.rebalance();
    }
    
    
    fn append_without_seam_check(&mut self, rope: Rope) {
        if self.grapheme_count_ == 0 {
            let mut r = rope;
            mem::swap(self, &mut r);
        }
        else if rope.grapheme_count_ == 0 {
            return;
        }
        else if self.tree_height > rope.tree_height {
            self.append_right(rope);
        }
        else {
            let mut rope = rope;
            mem::swap(self, &mut rope);
            self.append_left(rope);
        }
    }  
    
    
    fn append_right(&mut self, rope: Rope) {
        if self.tree_height <= rope.tree_height || self.is_leaf() {
            let mut temp_rope = Box::new(Rope::new());
            mem::swap(self, &mut (*temp_rope));
            self.data = RopeData::Branch(temp_rope, Box::new(rope));
        }
        else if let RopeData::Branch(_, ref mut right) = self.data {
            right.append_right(rope);
        }
        
        self.update_stats();
        self.rebalance();
    }
    
    
    fn append_left(&mut self, rope: Rope) {
        if self.tree_height <= rope.tree_height || self.is_leaf() {
            let mut temp_rope = Box::new(Rope::new());
            mem::swap(self, &mut (*temp_rope));
            self.data = RopeData::Branch(Box::new(rope), temp_rope);
        }
        else if let RopeData::Branch(ref mut left, _) = self.data {
            left.append_left(rope);
        }
        
        self.update_stats();
        self.rebalance();
    }
    
    
    /// Inserts the given text at the given char index.
    /// This is done without a seam check because it is recursive and
    /// would otherwise do a seam check at every recursive function call.
    /// Rope::insert_text_at_char_index() calls this, and then does the seam
    /// checks afterwards.
    fn insert_text_at_char_index_without_seam_check(&mut self, text: &str, pos: usize) {
        let mut leaf_insert = false;
        
        match self.data {
            // Find node for text to be inserted into
            RopeData::Branch(ref mut left, ref mut right) => {
                if pos < left.char_count_ {
                    left.insert_text_at_char_index(text, pos);
                }
                else {
                    right.insert_text_at_char_index(text, pos - left.char_count_);
                }
            },
            
            // Insert the text
            RopeData::Leaf(ref mut s_text) => {
                if grapheme_count_is_less_than(text, MAX_NODE_SIZE - self.grapheme_count_) {
                    // Simple case
                    insert_text_at_char_index(s_text, text, pos);
                }
                else {
                    // Special cases
                    leaf_insert = true;
                }
            },
        }
        
        // The special cases of inserting at a leaf node.
        // These have to be done outside of the match statement because
        // of the borrow checker, but logically they take place in the
        // RopeData::Leaf branch of the match statement above.
        if leaf_insert {
            // TODO: these special cases are currently prone to causing leaf
            // fragmentation.  Find ways to reduce that.
            if pos == 0 {
                let mut new_rope = Rope::new();
                mem::swap(self, &mut new_rope);
                self.data = RopeData::Branch(Box::new(Rope::from_str(text)), Box::new(new_rope));
            }
            else if pos == self.char_count_ {
                let mut new_rope = Rope::new();
                mem::swap(self, &mut new_rope);
                self.data = RopeData::Branch(Box::new(new_rope), Box::new(Rope::from_str(text)));
            }
            else {
                // Split the leaf node at the insertion point
                let mut node_l = Rope::new();
                let node_r = self.split_at_char_index(pos);
                mem::swap(self, &mut node_l);
                
                // Set the inserted text as the main node
                *self = Rope::from_str(text);
                
                // Append the left and right split nodes to either side of
                // the main node.
                self.append_left(node_l);
                self.append_right(node_r);
            }
        }
        
        self.update_stats();
        self.rebalance();
    }
    
    
    /// Removes the text between the given char indices.
    /// This is done without a seam check so that it can be used inside
    /// repair_grapheme_seam() without risk of unintended recursion.
    fn remove_text_between_char_indices_without_seam_check(&mut self, pos_a: usize, pos_b: usize) {
        // Bounds checks
        if pos_a > pos_b {
            panic!("Rope::remove_text_between_char_indices(): pos_a must be less than or equal to pos_b.");
        }
        if pos_b > self.char_count_ {
            panic!("Rope::remove_text_between_char_indices(): attempt to remove text after end of node text.");
        }
        
        match self.data {
            RopeData::Leaf(ref mut text) => {
                remove_text_between_char_indices(text, pos_a, pos_b);
            },
            
            RopeData::Branch(ref mut left, ref mut right) => {
                let lcc = left.char_count_;
                
                if pos_a < lcc {
                    left.remove_text_between_char_indices(pos_a, min(pos_b, lcc));
                }
                
                if pos_b > lcc {
                    right.remove_text_between_char_indices(pos_a - min(pos_a, lcc), pos_b - lcc);
                }
            }
        }
        
        self.update_stats();
        self.merge_if_too_small();
        self.rebalance();
    }


    /// Splits a leaf node into pieces if it's too large
    // TODO: find a way to do this that's more algorithmically efficient
    // if lots of splits need to happen.  This version ends up re-scanning
    // the text quite a lot, as well as doing quite a few unnecessary
    // allocations.
    fn split_if_too_large(&mut self) {
        if self.grapheme_count_ > MAX_NODE_SIZE && self.is_leaf() {
            
            // Calculate split position and how large the left and right
            // sides are going to be
            let split_pos = self.grapheme_count_ / 2;
            let new_gc_l = split_pos;
            let new_gc_r = self.grapheme_count_ - split_pos;

            // Do the split
            let mut nl = Box::new(Rope::new());
            let mut nr = Box::new(Rope::new());
            mem::swap(self, &mut (*nl));
            if let RopeData::Leaf(ref mut text) = nl.data {
                nr.data = RopeData::Leaf(split_string_at_grapheme_index(text, split_pos));
                text.shrink_to_fit();
            }
            
            // Recursively split
            nl.grapheme_count_ = new_gc_l;
            nr.grapheme_count_ = new_gc_r;
            nl.split_if_too_large();
            nr.split_if_too_large();
            
            // Update the new left and right node's stats
            nl.update_stats();
            nr.update_stats();
            
            // Create the new branch node with the new left and right nodes
            self.data = RopeData::Branch(nl, nr);
            self.update_stats();
        }
    }
    
    
    /// Merges a non-leaf node into a leaf node if it's too small
    fn merge_if_too_small(&mut self) {
        if self.grapheme_count_ < MIN_NODE_SIZE && !self.is_leaf() {
            let mut merged_text = String::new();
            
            if let RopeData::Branch(ref mut left, ref mut right) = self.data {
                // First, recursively merge the children
                left.merge_if_too_small();
                right.merge_if_too_small();
                
                // Then put their text into merged_text
                if let RopeData::Leaf(ref mut text) = left.data {
                    mem::swap(&mut merged_text, text);
                }        
                if let RopeData::Leaf(ref mut text) = right.data {
                    merged_text.push_str(&text[..]);
                }
            }
            
            // Make this a leaf node with merged_text as its data
            self.data = RopeData::Leaf(merged_text);
            self.tree_height = 1;
            // Don't need to update grapheme count, because it should be the
            // same as before.
        }
    }
    
    
    /// Rotates the tree under the node left
    fn rotate_left(&mut self) {
        let mut temp = Rope::new();
        
        if let RopeData::Branch(_, ref mut right) = self.data {
            mem::swap(&mut temp, &mut (**right));
            
            if let RopeData::Branch(ref mut left, _) = temp.data {   
                mem::swap(&mut (**left), &mut (**right));
            }
            else {
                panic!("Rope::rotate_left(): attempting to rotate node without branching right child.");
            }
        }
        else {
            panic!("Rope::rotate_left(): attempting to rotate leaf node.");
        }
        
        if let RopeData::Branch(ref mut left, _) = temp.data {
            mem::swap(&mut (**left), self);
            left.update_stats();
        }
        
        mem::swap(&mut temp, self);
        self.update_stats();
    }
    
    
    /// Rotates the tree under the node right
    fn rotate_right(&mut self) {
        let mut temp = Rope::new();
        
        if let RopeData::Branch(ref mut left, _) = self.data {
            mem::swap(&mut temp, &mut (**left));
            
            if let RopeData::Branch(_, ref mut right) = temp.data {   
                mem::swap(&mut (**right), &mut (**left));
            }
            else {
                panic!("Rope::rotate_right(): attempting to rotate node without branching left child.");
            }
        }
        else {
            panic!("Rope::rotate_right(): attempting to rotate leaf node.");
        }
        
        if let RopeData::Branch(_, ref mut right) = temp.data {
            mem::swap(&mut (**right), self);
            right.update_stats();
        }
        
        mem::swap(&mut temp, self);
        self.update_stats();
    }
    
    
    /// Balances the tree under this node.  Assumes that both the left and
    /// right sub-trees are themselves aleady balanced.
    /// Runs in time linear to the difference in height between the two
    /// sub-trees.  Thus worst-case is O(log N) time, and best-case is O(1)
    /// time.
    fn rebalance(&mut self) {
        let mut rot: isize = 0;
        
        if let RopeData::Branch(ref mut left, ref mut right) = self.data {
            let height_diff = (left.tree_height as isize) - (right.tree_height as isize);

            // Left side higher than right side
            if height_diff > 1 {
                let mut child_rot = false;
                if let RopeData::Branch(ref lc, ref rc) = left.data {
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
                if let RopeData::Branch(ref lc, ref rc) = right.data {
                    if lc.tree_height > rc.tree_height {
                        child_rot = true;
                    }
                }
                
                if child_rot {
                    right.rotate_right();
                }
                
                rot = -1;
            }
        }
        
        if rot == 1 {
            self.rotate_right();
            if let RopeData::Branch(_, ref mut right) = self.data {
                right.rebalance();
            }
        }
        else if rot == -1 {
            self.rotate_left();
            if let RopeData::Branch(ref mut left, _) = self.data {
                left.rebalance();
            }
        }
        
        self.update_stats();
    }
    
    
    /// Creates a grapheme iterator startin at the given char index.
    /// If the given char index starts in the middle of a grapheme,
    /// the grapheme is split and the part of the grapheme after the
    /// the char index is returned as the first grapheme.
    fn grapheme_iter_at_char_index<'a>(&'a self, index: usize) -> RopeGraphemeIter<'a> {
        let (char_i, mut chunk_iter) = self.chunk_iter_at_char_index(index);
        
        // Get the chunk string
        if let Some(text) = chunk_iter.next() {
            // Create the grapheme iter for the current node
            let byte_i = char_pos_to_byte_pos(text, index - char_i);
            let giter = UnicodeSegmentation::graphemes(&text[byte_i..], true);
            
            // Create the rope grapheme iter
            return RopeGraphemeIter {
                chunk_iter: chunk_iter,
                cur_chunk: giter,
                length: None,
            };
        }
        else {
            // No chunks, which means no text
            return RopeGraphemeIter {
                chunk_iter: chunk_iter,
                cur_chunk: UnicodeSegmentation::graphemes("", true),
                length: None,
            };
        };
    }
    
    
    /// Returns whether the given char index lies on a leaf node boundary.
    fn is_leaf_boundary(&self, index: usize) -> bool {
        if index == 0 || index == self.char_count_ {
            return true;
        }
        else {
            match self.data {
                RopeData::Leaf(_) => {
                    return false;
                },
                
                RopeData::Branch(ref left, ref right) => {
                    if index < left.char_count_ {
                        return left.is_leaf_boundary(index);
                    }
                    else {
                        return right.is_leaf_boundary(index - left.char_count_);
                    }
                }
            }
        }
    }
    
    
    fn append_to_leaf(&mut self, text: &str, index: usize) {
        match self.data {
            RopeData::Leaf(ref mut l_text) => {
                l_text.push_str(text);
            },
            
            RopeData::Branch(ref mut left, ref mut right) => {
                if index <= left.char_count_ {
                    left.append_to_leaf(text, index);
                }
                else {
                    right.append_to_leaf(text, index - left.char_count_);
                }
            }
        }
        
        self.update_stats();
    }
    
    
    /// Repairs an erroneous grapheme separation that can occur at
    /// leaf node boundaries.  The index given is the char index of the
    /// possible seam.
    fn repair_grapheme_seam(&mut self, index: usize) {
        if index == 0 || index == self.char_count_ {
            return;
        }
        
        let gi = self.char_index_to_grapheme_index(index);
        
        if self.is_leaf_boundary(index) && graphemes_are_mergeable(self.grapheme_at_index(gi-1), self.grapheme_at_index(gi)) {
            let c1 = self.grapheme_index_to_char_index(gi);
            let c2 = self.grapheme_index_to_char_index(gi + 1);
            
            // Get the grapheme on the right
            let mut s = String::new();
            s.push_str(self.grapheme_at_index(gi));
            
            // Append it to the left
            self.append_to_leaf(&s[..], index);
            
            // Remove the duplicate
            self.remove_text_between_char_indices_without_seam_check(c2, c2 + (c2 - c1));
        }
    }
    
    
    /// Tests if the rope adheres to the AVL balancing invariants.
    fn is_balanced(&self) -> bool {
        match self.data {
            RopeData::Leaf(_) => {
                return true;
            },
            
            RopeData::Branch(ref left, ref right) => {
                let mut diff = left.tree_height as isize - right.tree_height as isize;
                diff = if diff < 0 {-diff} else {diff};
                return (diff < 2) && left.is_balanced() && right.is_balanced();
            }
        }
    }
}




//=============================================================
// Rope iterators
//=============================================================

/// An iterator over a rope's string chunks
pub struct RopeChunkIter<'a> {
    node_stack: Vec<&'a Rope>,
}

impl<'a> Iterator for RopeChunkIter<'a> {
    type Item = &'a str;
    
    fn next(&mut self) -> Option<&'a str> {
        if let Some(next_chunk) = self.node_stack.pop() {
            loop {
                if let Option::Some(node) = self.node_stack.pop() {
                    match node.data {
                        RopeData::Leaf(_) => {
                            self.node_stack.push(node);
                            break;
                        },
                      
                        RopeData::Branch(ref left, ref right) => {
                            self.node_stack.push(&(**right));
                            self.node_stack.push(&(**left));
                            continue;
                        }
                    }
                }
                else {
                    break;
                }
            }
            
            if let RopeData::Leaf(ref text) = next_chunk.data {
                return Some(&text[..]);
            }
            else {
                unreachable!();
            }
        }
        else {
            return None;
        }
    }
}


// An iterator over a rope's chars
pub struct RopeCharIter<'a> {
    chunk_iter: RopeChunkIter<'a>,
    cur_chunk: Chars<'a>,
    length: Option<usize>,
}


impl<'a> Iterator for RopeCharIter<'a> {
    type Item = char;
    
    fn next(&mut self) -> Option<char> {
        if let Some(ref mut l) = self.length {
            if *l == 0 {
                return None;
            }
        }
        
        loop {
            if let Some(c) = self.cur_chunk.next() {
                if let Some(ref mut l) = self.length {
                    *l -= 1;
                }
                return Some(c);
            }
            else {   
                if let Some(s) = self.chunk_iter.next() {
                    self.cur_chunk = s.chars();
                    continue;
                }
                else {
                    return None;
                }
            }
        }
    }
}


/// An iterator over a rope's graphemes
pub struct RopeGraphemeIter<'a> {
    chunk_iter: RopeChunkIter<'a>,
    cur_chunk: Graphemes<'a>,
    length: Option<usize>, // Length in chars, not graphemes
}


impl<'a> Iterator for RopeGraphemeIter<'a> {
    type Item = &'a str;
    
    fn next(&mut self) -> Option<&'a str> {
        if let Some(ref mut l) = self.length {
            if *l == 0 {
                return None;
            }
        }
        
        loop {
            if let Some(g) = self.cur_chunk.next() {
                if let Some(ref mut l) = self.length {
                    let cc = char_count(g);
                    if *l >= cc {
                        *l -= char_count(g);
                        return Some(g);
                    }
                    else {
                        let bc = char_pos_to_byte_pos(g, *l);
                        *l = 0;
                        return Some(&g[..bc]);
                    }
                }
                else {
                    return Some(g);
                }
            }
            else {   
                if let Some(s) = self.chunk_iter.next() {
                    self.cur_chunk = UnicodeSegmentation::graphemes(s, true);
                    continue;
                }
                else {
                    return None;
                }
            }
        }
    }
}



/// An iterator over a rope's lines, returned as RopeSlice's
pub struct RopeLineIter<'a> {
    rope: &'a Rope,
    li: usize,
}


impl<'a> Iterator for RopeLineIter<'a> {
    type Item = RopeSlice<'a>;

    fn next(&mut self) -> Option<RopeSlice<'a>> {
        if self.li > self.rope.line_ending_count() {
            return None;
        }
        else {
            let a = self.rope.line_index_to_char_index(self.li);
            let b = if self.li < self.rope.line_ending_count() {
                self.rope.line_index_to_char_index(self.li+1)
            }
            else {
                self.rope.char_count()
            };
            
            self.li += 1;
            
            return Some(self.rope.slice(a, b));
        }
    }
}




//=============================================================
// Rope slice
//=============================================================

/// An immutable slice into a Rope
pub struct RopeSlice<'a> {
    rope: &'a Rope,
    start: usize,
    end: usize,
}


impl<'a> RopeSlice<'a> {
    pub fn char_count(&self) -> usize {
        self.end - self.start
    }
    

    pub fn grapheme_count(&self) -> usize {
        self.rope.grapheme_count_in_char_range(self.start, self.end)
    }
    
    
    pub fn char_iter(&self) -> RopeCharIter<'a> {
        self.rope.char_iter_between_indices(self.start, self.end)
    }
    
    pub fn char_iter_at_index(&self, pos: usize) -> RopeCharIter<'a> {
        assert!(pos <= self.char_count(), "RopeSlice::char_iter_at_index(): attempted to create iter starting beyond the end of the slice.");
        
        let a = self.start + pos;
        
        self.rope.char_iter_between_indices(a, self.end)
    }
    
    pub fn char_iter_between_indices(&self, pos_a: usize, pos_b: usize) -> RopeCharIter<'a> {
        assert!(pos_a <= pos_b, "RopeSlice::char_iter_between_indices(): pos_a must be less than or equal to pos_b.");
        assert!(pos_b <= self.char_count(), "RopeSlice::char_iter_between_indices(): attempted to create iter extending beyond the end of the slice.");
        
        let a = self.start + pos_a;
        let b = self.start + pos_b;
        
        self.rope.char_iter_between_indices(a, b)
    }
    
    
    pub fn grapheme_iter(&self) -> RopeGraphemeIter<'a> {
        self.grapheme_iter_at_index(0)
    }
    
    pub fn grapheme_iter_at_index(&self, pos: usize) -> RopeGraphemeIter<'a> {
        assert!(pos <= self.grapheme_count(), "RopeSlice::grapheme_iter_at_index(): attempted to create iter starting beyond the end of the slice.");
        
        let gs = self.rope.char_index_to_grapheme_index(self.start);
        let ca = self.rope.grapheme_index_to_char_index(gs + pos);
        
        let a = min(self.end, max(self.start, ca));
        
        let mut giter = self.rope.grapheme_iter_at_char_index(a);
        giter.length = Some(self.end - a);
        
        return giter;
    }
    
    pub fn grapheme_iter_between_indices(&self, pos_a: usize, pos_b: usize) -> RopeGraphemeIter<'a> {
        assert!(pos_a <= pos_b, "RopeSlice::grapheme_iter_between_indices(): pos_a must be less than or equal to pos_b.");
        assert!(pos_b <= self.grapheme_count(), "RopeSlice::grapheme_iter_between_indices(): attempted to create iter extending beyond the end of the slice.");
        
        let gs = self.rope.char_index_to_grapheme_index(self.start);
        let ca = self.rope.grapheme_index_to_char_index(gs + pos_a);
        let cb = self.rope.grapheme_index_to_char_index(gs + pos_b);
        
        let mut giter = self.rope.grapheme_iter_at_char_index(ca);
        giter.length = Some(cb - ca);
        
        return giter;
    }
    
    
    pub fn char_at_index(&self, index: usize) -> char {
        assert!(index < self.char_count(), "RopeSlice::char_at_index(): attempted to index beyond the end of the slice.");
        
        self.rope.char_at_index(self.start+index)
    }
    
    pub fn grapheme_at_index(&self, index: usize) -> &'a str {
        assert!(index < self.grapheme_count(), "RopeSlice::grapheme_at_index(): attempted to index beyond the end of the slice.");
        
        let gs = self.rope.char_index_to_grapheme_index(self.start);
        let gi = gs + index;
        let cs = self.rope.grapheme_index_to_char_index(gi);
        let ce = self.rope.grapheme_index_to_char_index(gi+1);
        
        let g = self.rope.grapheme_at_index(gi);
        
        if cs >= self.start && ce <= self.end {
            // Easy case
            return g;
        }
        else {
            // Hard case: partial graphemes
            let shave_a = if cs < self.start { self.start - cs} else { 0 };
            let shave_b = if ce > self.end { ce - self.end } else { 0 };
            
            let cc = char_count(g);
            
            let a = char_pos_to_byte_pos(g, shave_a);
            let b = char_pos_to_byte_pos(g, cc - shave_b);
            
            return &g[a..b];
        }
    }
    
    
    pub fn slice(&self, pos_a: usize, pos_b: usize) -> RopeSlice<'a> {
        assert!(pos_a <= pos_b, "RopeSlice::slice(): pos_a must be less than or equal to pos_b.");
        assert!(pos_b <= self.char_count(), "RopeSlice::slice(): attempted to create slice extending beyond the end of this slice.");
        
        let a = self.start + pos_a;
        let b = self.start + pos_b;
        
        RopeSlice {
            rope: self.rope,
            start: a,
            end: b,
        }
    }
}
