use std::cmp::{min, max};
use std::mem;
use std::str::Graphemes;
use std::ops::Index;
use string_utils::{
    grapheme_and_line_ending_count,
    grapheme_count_is_less_than,
    insert_text_at_grapheme_index,
    remove_text_between_grapheme_indices,
    split_string_at_grapheme_index,
    is_line_ending,
    LineEnding,
    str_to_line_ending,
};

pub const MIN_NODE_SIZE: usize = 64;
pub const MAX_NODE_SIZE: usize = MIN_NODE_SIZE * 2;


/// A rope data structure for storing text in a format that is efficient
/// for insertion and removal even for extremely large strings.
#[derive(Debug)]
pub struct Rope {
    data: RopeData,
    grapheme_count_: usize,
    line_ending_count: usize,
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
            grapheme_count_: 0,
            line_ending_count: 0,
            tree_height: 1,
        }
    }
    

    /// Creates a new rope from a string slice    
    pub fn new_from_str(s: &str) -> Rope {
        let mut rope_stack: Vec<Rope> = Vec::new();
        
        let mut s1 = s;
        loop {
            // Get the next chunk of the string to add
            let mut byte_i = 0;
            let mut le_count = 0;
            let mut g_count = 0;
            for (bi, g) in s1.grapheme_indices(true) {
                byte_i = bi + g.len();
                g_count += 1;
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
                data: RopeData::Leaf(String::from_str(chunk)),
                grapheme_count_: g_count,
                line_ending_count: le_count,
                tree_height: 1,
            });
            
            // Do merges
            loop {
                let rsl = rope_stack.len();
                if rsl > 1 && rope_stack[rsl-2].tree_height <= rope_stack[rsl-1].tree_height {
                    let right = Box::new(rope_stack.pop().unwrap());
                    let left = Box::new(rope_stack.pop().unwrap());
                    let h = max(left.tree_height, right.tree_height) + 1;
                    let lc = left.line_ending_count + right.line_ending_count;
                    let gc = left.grapheme_count_ + right.grapheme_count_;
                    rope_stack.push(Rope {
                        data: RopeData::Branch(left, right),
                        grapheme_count_: gc,
                        line_ending_count: lc,
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
    
    pub fn new_from_str_with_count(s: &str, g_count: usize, le_count: usize) -> Rope {
        if g_count <= MAX_NODE_SIZE {
            Rope {
                data: RopeData::Leaf(String::from_str(s)),
                grapheme_count_: g_count,
                line_ending_count: le_count,
                tree_height: 1,
            }
        }
        else {
            Rope::new_from_str(s)
        }
    }
    
    /// Creates a new rope from a string, consuming the string
    pub fn new_from_string(s: String) -> Rope {
        // TODO: special case short strings?
        Rope::new_from_str(s.as_slice())
    }
    
    pub fn grapheme_count(&self) -> usize {
        return self.grapheme_count_;
    }
    
    pub fn line_count(&self) -> usize {
        return self.line_ending_count + 1;
    }
    
    
    /// Returns the grapheme index at the start of the given line index.
    pub fn line_index_to_grapheme_index(&self, li: usize) -> usize {
        // Bounds check
        if li > self.line_ending_count {
            panic!("Rope::line_index_to_grapheme_index: line index is out of bounds.");
        }
        
        // Special case for the beginning of the rope
        if li == 0 {
            return 0;
        }
        
        // General cases
        match self.data {
            RopeData::Leaf(ref text) => {
                let mut gi = 0;
                let mut lei = 0;
                for g in text.as_slice().graphemes(true) {
                    gi += 1;
                    if is_line_ending(g) {
                        lei += 1;
                    }
                    if lei == li {
                        break;
                    }
                }
                return gi;
            },
            
            RopeData::Branch(ref left, ref right) => {
                if li <= left.line_ending_count {
                    return left.line_index_to_grapheme_index(li);
                }
                else {
                    return right.line_index_to_grapheme_index(li - left.line_ending_count) + left.grapheme_count_;
                }
            },
        }
    }
    
    
    /// Returns the index of the line that the given grapheme index is on.
    pub fn grapheme_index_to_line_index(&self, pos: usize) -> usize {
        match self.data {
            RopeData::Leaf(ref text) => {
                let mut gi = 0;
                let mut lei = 0;
                for g in text.as_slice().graphemes(true) {
                    if gi == pos {
                        break;
                    }
                    gi += 1;
                    if is_line_ending(g) {
                        lei += 1;
                    }
                }
                return lei;
            },
            
            RopeData::Branch(ref left, ref right) => {
                if pos < left.grapheme_count_ {
                    return left.grapheme_index_to_line_index(pos);
                }
                else {
                    return right.grapheme_index_to_line_index(pos - left.grapheme_count_) + left.line_ending_count;
                }
            },
        }
    }
    
    
    /// Converts a grapheme index into a line number and grapheme-column
    /// number.
    ///
    /// If the index is off the end of the text, returns the line and column
    /// number of the last valid text position.
    pub fn grapheme_index_to_line_col(&self, pos: usize) -> (usize, usize) {
        let p = min(pos, self.grapheme_count_);
        let line = self.grapheme_index_to_line_index(p);
        let line_pos = self.line_index_to_grapheme_index(line);
        return (line, p - line_pos);
    }
    
    
    /// Converts a line number and grapheme-column number into a grapheme
    /// index.
    ///
    /// If the column number given is beyond the end of the line, returns the
    /// index of the line's last valid position.  If the line number given is
    /// beyond the end of the buffer, returns the index of the buffer's last
    /// valid position.
    pub fn line_col_to_grapheme_index(&self, pos: (usize, usize)) -> usize {
        if pos.0 <= self.line_ending_count {
            let l_begin_pos = self.line_index_to_grapheme_index(pos.0);
            
            let l_end_pos = if pos.0 < self.line_ending_count {
                self.line_index_to_grapheme_index(pos.0 + 1) - 1
            }
            else {
                self.grapheme_count_
            };
                
            return min(l_begin_pos + pos.1, l_end_pos);
        }
        else {
            return self.grapheme_count_;
        }
    }
    
    
    pub fn grapheme_at_index<'a>(&'a self, index: usize) -> &'a str {
        &self[index]
    }
    
    
    /// Inserts the given text at the given grapheme index.
    /// For small lengths of 'text' runs in O(log N) time.
    /// For large lengths of 'text', dunno.  But it seems to perform
    /// sub-linearly, at least.
    pub fn insert_text_at_grapheme_index(&mut self, text: &str, pos: usize) {
        let mut leaf_insert = false;
        
        match self.data {
            // Find node for text to be inserted into
            RopeData::Branch(ref mut left, ref mut right) => {
                if pos < left.grapheme_count_ {
                    left.insert_text_at_grapheme_index(text, pos);
                }
                else {
                    right.insert_text_at_grapheme_index(text, pos - left.grapheme_count_);
                }
            },
            
            // Insert the text
            RopeData::Leaf(ref mut s_text) => {
                if grapheme_count_is_less_than(text, MAX_NODE_SIZE - self.grapheme_count_ + 1) {
                    // Simple case
                    insert_text_at_grapheme_index(s_text, text, pos);
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
                self.data = RopeData::Branch(Box::new(Rope::new_from_str(text)), Box::new(new_rope));
            }
            else if pos == self.grapheme_count_ {
                let mut new_rope = Rope::new();
                mem::swap(self, &mut new_rope);
                self.data = RopeData::Branch(Box::new(new_rope), Box::new(Rope::new_from_str(text)));
            }
            else {
                // Split the leaf node at the insertion point
                let mut node_l = Rope::new();
                let node_r = self.split(pos);
                mem::swap(self, &mut node_l);
                
                // Set the inserted text as the main node
                *self = Rope::new_from_str(text);
                
                // Append the left and right split nodes to either side of
                // the main node.
                self.append_left(node_l);
                self.append_right(node_r);
            }
        }
        
        self.update_stats();
        self.rebalance();
    }
    
    
    /// Removes the text between grapheme indices pos_a and pos_b.
    /// For small distances between pos_a and pos_b runs in O(log N) time.
    /// For large distances, dunno.  If it becomes a performance bottleneck,
    /// can special-case that to two splits and an append, which are all
    /// O(log N).
    pub fn remove_text_between_grapheme_indices(&mut self, pos_a: usize, pos_b: usize) {
        // Bounds checks
        if pos_a > pos_b {
            panic!("Rope::remove_text_between_grapheme_indices(): pos_a must be less than or equal to pos_b.");
        }
        if pos_b > self.grapheme_count_ {
            panic!("Rope::remove_text_between_grapheme_indices(): attempt to remove text after end of node text.");
        }
        
        match self.data {
            RopeData::Leaf(ref mut text) => {
                remove_text_between_grapheme_indices(text, pos_a, pos_b);
            },
            
            RopeData::Branch(ref mut left, ref mut right) => {
                let lgc = left.grapheme_count_;
                
                if pos_a < lgc {
                    left.remove_text_between_grapheme_indices(pos_a, min(pos_b, lgc));
                }
                
                if pos_b > lgc {
                    right.remove_text_between_grapheme_indices(pos_a - min(pos_a, lgc), pos_b - lgc);
                }
            }
        }
        
        self.update_stats();
        self.merge_if_too_small();
        self.rebalance();
    }
    
    /// Splits a rope into two pieces from the given grapheme index.
    /// The first piece remains in this rope, the second piece is returned
    /// as a new rope.
    /// Runs in O(log N) time.
    pub fn split(&mut self, pos: usize) -> Rope {
        let mut left = Rope::new();
        let mut right = Rope::new();
        
        self.split_recursive(pos, &mut left, &mut right);
        
        mem::swap(self, &mut left);
        return right;
    }

    /// Appends another rope to the end of this one, consuming the other rope.
    /// Runs in O(log N) time.
    pub fn append(&mut self, rope: Rope) {
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
        self.chunk_iter_at_index(0).1
    }
    
    
    /// Creates a chunk iter starting at the chunk containing the given
    /// grapheme index.  Returns the chunk and its starting grapheme index.
    pub fn chunk_iter_at_index<'a>(&'a self, index: usize) -> (usize, RopeChunkIter<'a>) {
        let mut node_stack: Vec<&'a Rope> = Vec::new();
        let mut cur_node = self;
        let mut grapheme_i = index;
        
        // Find the right rope node, and populate the stack at the same time
        loop {
            match cur_node.data {
                RopeData::Leaf(_) => {
                    node_stack.push(cur_node);
                    break;
                },
                
                RopeData::Branch(ref left, ref right) => {
                    if grapheme_i < left.grapheme_count_ {
                        node_stack.push(&(**right));
                        cur_node = &(**left);
                    }
                    else {
                        cur_node = &(**right);
                        grapheme_i -= left.grapheme_count_;
                    }
                }
            }
        }
        
        (index - grapheme_i, RopeChunkIter {node_stack: node_stack})
    }
    
    
    /// Creates an iterator at the first grapheme of the rope
    pub fn grapheme_iter<'a>(&'a self) -> RopeGraphemeIter<'a> {
        self.grapheme_iter_at_index(0)
    }
    
    
    /// Creates an iterator at the given grapheme index
    pub fn grapheme_iter_at_index<'a>(&'a self, index: usize) -> RopeGraphemeIter<'a> {
        let (grapheme_i, mut chunk_iter) = self.chunk_iter_at_index(index);
        
        // Create the grapheme iter for the current node
        let mut giter = if let Some(text) = chunk_iter.next() {
            text.as_slice().graphemes(true)
        }
        else {
            unreachable!()
        };
        
        // Get to the right spot in the iter
        for _ in grapheme_i..index {
            giter.next();
        }
        
        // Create the rope grapheme iter
        return RopeGraphemeIter {
            chunk_iter: chunk_iter,
            cur_chunk: giter,
            length: None,
        };
    }
    
    
    /// Creates an iterator that starts a pos_a and stops just before pos_b.
    pub fn grapheme_iter_between_indices<'a>(&'a self, pos_a: usize, pos_b: usize) -> RopeGraphemeIter<'a> {
        let mut iter = self.grapheme_iter_at_index(pos_a);
        iter.length = Some(pos_b - pos_a);
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
        RopeLineIter {
            rope: self,
            li: index,
        }
    }
    
    
    pub fn slice<'a>(&'a self, pos_a: usize, pos_b: usize) -> RopeSlice<'a> {
        let a = pos_a;
        let b = min(self.grapheme_count_, pos_b);
        
        RopeSlice {
            rope: self,
            start: a,
            end: b,
        }
    }
    
    
    // Creates a graphviz document of the Rope's structure, and returns
    // it as a string.  For debugging purposes.
    pub fn to_graphviz(&self) -> String {
        let mut text = String::from_str("digraph {\n");
        self.to_graphviz_recursive(&mut text, String::from_str("s"));
        text.push_str("}\n");
        return text;
    }
    
    
    //================================================================
    // Private utility functions
    //================================================================
    
    
    fn to_graphviz_recursive(&self, text: &mut String, name: String) {
        match self.data {
            RopeData::Leaf(_) => {
                text.push_str(format!("{} [label=\"gc={}\\nlec={}\"];\n", name, self.grapheme_count_, self.line_ending_count).as_slice());
            },
            
            RopeData::Branch(ref left, ref right) => {
                let mut lname = name.clone();
                let mut rname = name.clone();
                lname.push('l');
                rname.push('r');
                text.push_str(format!("{} [shape=box, label=\"h={}\\ngc={}\\nlec={}\"];\n", name, self.tree_height, self.grapheme_count_, self.line_ending_count).as_slice());
                text.push_str(format!("{} -> {{ {} {} }};\n", name, lname, rname).as_slice());
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
                let (gc, lec) = grapheme_and_line_ending_count(text);
                self.grapheme_count_ = gc;
                self.line_ending_count = lec;
                self.tree_height = 1;
            },
            
            RopeData::Branch(ref left, ref right) => {
                self.grapheme_count_ = left.grapheme_count_ + right.grapheme_count_;
                self.line_ending_count = left.line_ending_count + right.line_ending_count;
                self.tree_height = max(left.tree_height, right.tree_height) + 1;
            }
        }
    }
    
    
    fn split_recursive(&mut self, pos: usize, left: &mut Rope, right: &mut Rope) {
        match self.data {
            RopeData::Leaf(ref text) => {
                // Split the text into two new nodes
                let mut l_text = text.clone();
                let r_text = split_string_at_grapheme_index(&mut l_text, pos);
                let new_rope_l = Rope::new_from_string(l_text);
                let mut new_rope_r = Rope::new_from_string(r_text);
                
                // Append the nodes to their respective sides
                left.append(new_rope_l);
                mem::swap(right, &mut new_rope_r);
                right.append(new_rope_r);
            },
            
            RopeData::Branch(ref mut left_b, ref mut right_b) => {
                let mut l = Rope::new();
                let mut r = Rope::new();
                mem::swap(&mut **left_b, &mut l);
                mem::swap(&mut **right_b, &mut r);
                
                // Split is on left side
                if pos < l.grapheme_count_ {
                    // Append the right split to the right side
                    mem::swap(right, &mut r);
                    right.append(r);
                    
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
                    let new_pos = pos - l.grapheme_count_;
                    left.append(l);
                    
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
                    merged_text.push_str(text.as_slice());
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


// Direct indexing to graphemes in the rope
impl Index<usize> for Rope {
    type Output = str;
    
    fn index<'a>(&'a self, index: &usize) -> &'a str {
        if *index >= self.grapheme_count() {
            panic!("Rope::Index: attempting to fetch grapheme that outside the bounds of the text.");
        }
        
        match self.data {
            RopeData::Leaf(ref text) => {
                let mut i: usize = 0;
                for g in text.graphemes(true) {
                    if i == *index {
                        return &g;
                    }
                    i += 1;
                }
                unreachable!();
            },
            
            RopeData::Branch(ref left, ref right) => {
                if *index < left.grapheme_count() {
                    return &left[*index];
                }
                else {
                    return &right[*index - left.grapheme_count()];
                }
            },
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
                return Some(text.as_slice());
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



/// An iterator over a rope's graphemes
pub struct RopeGraphemeIter<'a> {
    chunk_iter: RopeChunkIter<'a>,
    cur_chunk: Graphemes<'a>,
    length: Option<usize>,
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
                    *l -= 1;
                }
                return Some(g);
            }
            else {   
                if let Some(s) = self.chunk_iter.next() {
                    self.cur_chunk = s.graphemes(true);
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
        if self.li >= self.rope.line_count() {
            return None;
        }
        else {
            let a = self.rope.line_index_to_grapheme_index(self.li);
            let b = if self.li+1 < self.rope.line_count() {
                self.rope.line_index_to_grapheme_index(self.li+1)
            }
            else {
                self.rope.grapheme_count()
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
    pub fn grapheme_count(&self) -> usize {
        self.end - self.start
    }
    
    
    pub fn grapheme_iter(&self) -> RopeGraphemeIter<'a> {
        self.rope.grapheme_iter_between_indices(self.start, self.end)
    }
    
    pub fn grapheme_iter_at_index(&self, pos: usize) -> RopeGraphemeIter<'a> {
        let a = min(self.end, self.start + pos);
        
        self.rope.grapheme_iter_between_indices(a, self.end)
    }
    
    pub fn grapheme_iter_between_indices(&self, pos_a: usize, pos_b: usize) -> RopeGraphemeIter<'a> {
        let a = min(self.end, self.start + pos_a);
        let b = min(self.end, self.start + pos_b);
        
        self.rope.grapheme_iter_between_indices(a, b)
    }
    
    
    pub fn grapheme_at_index(&self, index: usize) -> &'a str {
        &self.rope[self.start+index]
    }
    
    
    /// Convenience function for when the slice represents a line
    pub fn ending(&self) -> LineEnding {
        if self.grapheme_count() > 0 {
            let g = self.grapheme_at_index(self.grapheme_count() - 1);
            return str_to_line_ending(g);
        }
        else {
            return LineEnding::None;
        }
    }
    
    
    pub fn slice(&self, pos_a: usize, pos_b: usize) -> RopeSlice<'a> {
        let a = min(self.end, self.start + pos_a);
        let b = min(self.end, self.start + pos_b);
        
        RopeSlice {
            rope: self.rope,
            start: a,
            end: b,
        }
    }
}




//===================================================================
// Unit test
//===================================================================

#[cfg(test)]
mod tests {
    #![allow(unused_imports)]
    use std::iter;
    use super::{Rope, RopeData, RopeGraphemeIter, MAX_NODE_SIZE};
    use std::old_path::Path;
    use std::old_io::fs::File;
    use std::old_io::BufferedWriter;


    #[test]
    fn new_1() {
        let rope = Rope::new();
        let mut iter = rope.grapheme_iter();
        
        assert_eq!(None, iter.next());
    }
    
    
    #[test]
    fn new_2() {
        let rope = Rope::new_from_str("Hello world!");
        let mut iter = rope.grapheme_iter();
        
        assert_eq!(Some("H"), iter.next());
        assert_eq!(Some("e"), iter.next());
        assert_eq!(Some("l"), iter.next());
        assert_eq!(Some("l"), iter.next());
        assert_eq!(Some("o"), iter.next());
        assert_eq!(Some(" "), iter.next());
        assert_eq!(Some("w"), iter.next());
        assert_eq!(Some("o"), iter.next());
        assert_eq!(Some("r"), iter.next());
        assert_eq!(Some("l"), iter.next());
        assert_eq!(Some("d"), iter.next());
        assert_eq!(Some("!"), iter.next());
        assert_eq!(None, iter.next());
    }
    
    
    #[test]
    fn new_3() {
        let s = String::from_str("Hello world!");
        let rope = Rope::new_from_string(s);
        let mut iter = rope.grapheme_iter();
        
        assert_eq!(Some("H"), iter.next());
        assert_eq!(Some("e"), iter.next());
        assert_eq!(Some("l"), iter.next());
        assert_eq!(Some("l"), iter.next());
        assert_eq!(Some("o"), iter.next());
        assert_eq!(Some(" "), iter.next());
        assert_eq!(Some("w"), iter.next());
        assert_eq!(Some("o"), iter.next());
        assert_eq!(Some("r"), iter.next());
        assert_eq!(Some("l"), iter.next());
        assert_eq!(Some("d"), iter.next());
        assert_eq!(Some("!"), iter.next());
        assert_eq!(None, iter.next());
    }
    
    
    #[test]
    fn new_4() {
        let rope = Rope::new_from_str(String::from_utf8(vec!['c' as u8; 1 + MAX_NODE_SIZE * 53]).unwrap().as_slice());
        
        assert!(rope.is_balanced());
    }
    
    
    #[test]
    fn index() {
        let rope = Rope::new_from_str("Hel世界lo world!");
        
        assert_eq!("H", &rope[0]);
        assert_eq!("界", &rope[4]);
    }
    
    
    #[test]
    fn slice_1() {
        let rope = Rope::new_from_str("Hello everyone!  How are you doing, eh?");
        let s = rope.slice(0, 15);
        
        let mut iter = s.grapheme_iter();
        
        assert_eq!(s.grapheme_count(), 15);
        assert!(Some("H") == iter.next());
        assert!(Some("e") == iter.next());
        assert!(Some("l") == iter.next());
        assert!(Some("l") == iter.next());
        assert!(Some("o") == iter.next());
        assert!(Some(" ") == iter.next());
        assert!(Some("e") == iter.next());
        assert!(Some("v") == iter.next());
        assert!(Some("e") == iter.next());
        assert!(Some("r") == iter.next());
        assert!(Some("y") == iter.next());
        assert!(Some("o") == iter.next());
        assert!(Some("n") == iter.next());
        assert!(Some("e") == iter.next());
        assert!(Some("!") == iter.next());
        assert!(None == iter.next());
    }
    
    
    #[test]
    fn slice_2() {
        let rope = Rope::new_from_str("Hello everyone!  How are you doing, eh?");
        let s = rope.slice(6, 20);
        
        let mut iter = s.grapheme_iter();
        
        assert_eq!(s.grapheme_count(), 14);
        assert!(Some("e") == iter.next());
        assert!(Some("v") == iter.next());
        assert!(Some("e") == iter.next());
        assert!(Some("r") == iter.next());
        assert!(Some("y") == iter.next());
        assert!(Some("o") == iter.next());
        assert!(Some("n") == iter.next());
        assert!(Some("e") == iter.next());
        assert!(Some("!") == iter.next());
        assert!(Some(" ") == iter.next());
        assert!(Some(" ") == iter.next());
        assert!(Some("H") == iter.next());
        assert!(Some("o") == iter.next());
        assert!(Some("w") == iter.next());
        assert!(None == iter.next());
    }
    
    
    #[test]
    fn slice_3() {
        let rope = Rope::new_from_str("Hello everyone!  How are you doing, eh?");
        let s = rope.slice(21, 39);
        
        let mut iter = s.grapheme_iter();
        
        assert_eq!(s.grapheme_count(), 18);
        assert!(Some("a") == iter.next());
        assert!(Some("r") == iter.next());
        assert!(Some("e") == iter.next());
        assert!(Some(" ") == iter.next());
        assert!(Some("y") == iter.next());
        assert!(Some("o") == iter.next());
        assert!(Some("u") == iter.next());
        assert!(Some(" ") == iter.next());
        assert!(Some("d") == iter.next());
        assert!(Some("o") == iter.next());
        assert!(Some("i") == iter.next());
        assert!(Some("n") == iter.next());
        assert!(Some("g") == iter.next());
        assert!(Some(",") == iter.next());
        assert!(Some(" ") == iter.next());
        assert!(Some("e") == iter.next());
        assert!(Some("h") == iter.next());
        assert!(Some("?") == iter.next());
        assert!(None == iter.next());
    }
    
    
    #[test]
    fn slice_4() {
        let rope = Rope::new_from_str("Hello everyone!  How are you doing, eh?");
        let s = rope.slice(21, 40);
        
        let mut iter = s.grapheme_iter();
        
        assert_eq!(s.grapheme_count(), 18);
        assert!(Some("a") == iter.next());
        assert!(Some("r") == iter.next());
        assert!(Some("e") == iter.next());
        assert!(Some(" ") == iter.next());
        assert!(Some("y") == iter.next());
        assert!(Some("o") == iter.next());
        assert!(Some("u") == iter.next());
        assert!(Some(" ") == iter.next());
        assert!(Some("d") == iter.next());
        assert!(Some("o") == iter.next());
        assert!(Some("i") == iter.next());
        assert!(Some("n") == iter.next());
        assert!(Some("g") == iter.next());
        assert!(Some(",") == iter.next());
        assert!(Some(" ") == iter.next());
        assert!(Some("e") == iter.next());
        assert!(Some("h") == iter.next());
        assert!(Some("?") == iter.next());
        assert!(None == iter.next());
    }
    
    
    #[test]
    fn slice_5() {
        let rope = Rope::new_from_str("Hello everyone!  How are you doing, eh?");
        let s = rope.slice(21, 40);
        let s2 = s.slice(3, 10);
        
        let mut iter = s2.grapheme_iter();
        
        assert_eq!(s.grapheme_count(), 18);
        assert!(Some(" ") == iter.next());
        assert!(Some("y") == iter.next());
        assert!(Some("o") == iter.next());
        assert!(Some("u") == iter.next());
        assert!(Some(" ") == iter.next());
        assert!(Some("d") == iter.next());
        assert!(Some("o") == iter.next());
        assert!(None == iter.next());
    }
    
    
    #[test]
    fn slice_6() {
        let rope = Rope::new_from_str("Hello everyone!  How are you doing, eh?");
        let s = rope.slice(15, 39);
        
        let mut iter = s.grapheme_iter_between_indices(0, 24);
        
        assert!(Some(" ") == iter.next());
        assert!(Some(" ") == iter.next());
        assert!(Some("H") == iter.next());
        assert!(Some("o") == iter.next());
        assert!(Some("w") == iter.next());
        assert!(Some(" ") == iter.next());
        assert!(Some("a") == iter.next());
        assert!(Some("r") == iter.next());
        assert!(Some("e") == iter.next());
        assert!(Some(" ") == iter.next());
        assert!(Some("y") == iter.next());
        assert!(Some("o") == iter.next());
        assert!(Some("u") == iter.next());
        assert!(Some(" ") == iter.next());
        assert!(Some("d") == iter.next());
        assert!(Some("o") == iter.next());
        assert!(Some("i") == iter.next());
        assert!(Some("n") == iter.next());
        assert!(Some("g") == iter.next());
        assert!(Some(",") == iter.next());
        assert!(Some(" ") == iter.next());
        assert!(Some("e") == iter.next());
        assert!(Some("h") == iter.next());
        assert!(Some("?") == iter.next());
        assert!(None == iter.next());
    }
    
    
    #[test]
    fn slice_7() {
        let rope = Rope::new_from_str("Hello everyone!  How are you doing, eh?");
        let s = rope.slice(15, 39);
        
        let mut iter = s.grapheme_iter_between_indices(10, 20);
        
        assert!(Some("y") == iter.next());
        assert!(Some("o") == iter.next());
        assert!(Some("u") == iter.next());
        assert!(Some(" ") == iter.next());
        assert!(Some("d") == iter.next());
        assert!(Some("o") == iter.next());
        assert!(Some("i") == iter.next());
        assert!(Some("n") == iter.next());
        assert!(Some("g") == iter.next());
        assert!(Some(",") == iter.next());
        assert!(None == iter.next());
    }
    

    #[test]
    fn line_index_to_grapheme_index_1() {
        let rope = Rope::new_from_str("Hello\nworld!\n");
        
        assert_eq!(rope.line_index_to_grapheme_index(0), 0);
        assert_eq!(rope.line_index_to_grapheme_index(1), 6);
        assert_eq!(rope.line_index_to_grapheme_index(2), 13);
    }
    
    
    #[test]
    fn line_index_to_grapheme_index_2() {
        let rope = Rope::new_from_str("Hi\nthere\npeople\nof\nthe\nworld!");
        
        assert_eq!(rope.line_index_to_grapheme_index(0), 0);
        assert_eq!(rope.line_index_to_grapheme_index(1), 3);
        assert_eq!(rope.line_index_to_grapheme_index(2), 9);
        assert_eq!(rope.line_index_to_grapheme_index(3), 16);
        assert_eq!(rope.line_index_to_grapheme_index(4), 19);
        assert_eq!(rope.line_index_to_grapheme_index(5), 23);
    }
    
    
    #[test]
    fn grapheme_index_to_line_index_1() {
        let rope = Rope::new_from_str("Hello\nworld!\n");
        
        assert_eq!(rope.grapheme_index_to_line_index(0), 0);
        assert_eq!(rope.grapheme_index_to_line_index(1), 0);
        assert_eq!(rope.grapheme_index_to_line_index(5), 0);
        assert_eq!(rope.grapheme_index_to_line_index(6), 1);
        assert_eq!(rope.grapheme_index_to_line_index(12), 1);
        assert_eq!(rope.grapheme_index_to_line_index(13), 2);
    }
    
    
    #[test]
    fn grapheme_index_to_line_index_2() {
        let rope = Rope::new_from_str("Hi\nthere\npeople\nof\nthe\nworld!");
        
        assert_eq!(rope.grapheme_index_to_line_index(0), 0);
        assert_eq!(rope.grapheme_index_to_line_index(2), 0);
        assert_eq!(rope.grapheme_index_to_line_index(3), 1);
        assert_eq!(rope.grapheme_index_to_line_index(8), 1);
        assert_eq!(rope.grapheme_index_to_line_index(9), 2);
        assert_eq!(rope.grapheme_index_to_line_index(15), 2);
        assert_eq!(rope.grapheme_index_to_line_index(16), 3);
        assert_eq!(rope.grapheme_index_to_line_index(18), 3);
        assert_eq!(rope.grapheme_index_to_line_index(19), 4);
        assert_eq!(rope.grapheme_index_to_line_index(22), 4);
        assert_eq!(rope.grapheme_index_to_line_index(23), 5);
        assert_eq!(rope.grapheme_index_to_line_index(29), 5);
    }
    
    
    #[test]
    fn grapheme_index_to_line_col_1() {
        let rope = Rope::new_from_str("Hello\nworld!\n");
        
        assert_eq!(rope.grapheme_index_to_line_col(0), (0,0));
        assert_eq!(rope.grapheme_index_to_line_col(5), (0,5));
        assert_eq!(rope.grapheme_index_to_line_col(6), (1,0));
        assert_eq!(rope.grapheme_index_to_line_col(12), (1,6));
        assert_eq!(rope.grapheme_index_to_line_col(13), (2,0));
        assert_eq!(rope.grapheme_index_to_line_col(14), (2,0));
    }
    
    
    #[test]
    fn line_col_to_grapheme_index_1() {
        let rope = Rope::new_from_str("Hello\nworld!\n");
        
        assert_eq!(rope.line_col_to_grapheme_index((0,0)), 0);
        assert_eq!(rope.line_col_to_grapheme_index((0,5)), 5);
        assert_eq!(rope.line_col_to_grapheme_index((0,6)), 5);
        
        assert_eq!(rope.line_col_to_grapheme_index((1,0)), 6);
        assert_eq!(rope.line_col_to_grapheme_index((1,6)), 12);
        assert_eq!(rope.line_col_to_grapheme_index((1,7)), 12);
        
        assert_eq!(rope.line_col_to_grapheme_index((2,0)), 13);
        assert_eq!(rope.line_col_to_grapheme_index((2,1)), 13);        
    }
    
    
    #[test]
    fn to_string() {
        let rope = Rope::new_from_str("Hello there good people of the world!");
        let s = rope.to_string();
        
        assert_eq!("Hello there good people of the world!", s.as_slice());
    }
    
    
    #[test]
    fn split_1() {
        let mut rope1 = Rope::new_from_str("Hello there good people of the world!");
        
        //let mut f1 = BufferedWriter::new(File::create(&Path::new("yar1.gv")).unwrap());
        //f1.write_str(rope1.to_graphviz().as_slice());
                
        let rope2 = rope1.split(18);

        //let mut f2 = BufferedWriter::new(File::create(&Path::new("yar2.gv")).unwrap());
        //f2.write_str(rope1.to_graphviz().as_slice());
        //f2.write_str(rope2.to_graphviz().as_slice());
        
        assert!(rope1.is_balanced());
        assert!(rope2.is_balanced());
        assert_eq!("Hello there good p", rope1.to_string().as_slice());
        assert_eq!("eople of the world!", rope2.to_string().as_slice());
    }
    
    
    #[test]
    fn split_2() {
        let mut rope1 = Rope::new_from_str("Hello there good people of the world!");
        
        //let mut f1 = BufferedWriter::new(File::create(&Path::new("yar1.gv")).unwrap());
        //f1.write_str(rope1.to_graphviz().as_slice());
                
        let rope2 = rope1.split(31);

        //let mut f2 = BufferedWriter::new(File::create(&Path::new("yar2.gv")).unwrap());
        //f2.write_str(rope1.to_graphviz().as_slice());
        //f2.write_str(rope2.to_graphviz().as_slice());
        
        assert!(rope1.is_balanced());
        assert!(rope2.is_balanced());
        assert_eq!("Hello there good people of the ", rope1.to_string().as_slice());
        assert_eq!("world!", rope2.to_string().as_slice());
    }
    
    
    #[test]
    fn split_3() {
        let mut rope1 = Rope::new_from_str("Hello there good people of the world!");
        
        //let mut f1 = BufferedWriter::new(File::create(&Path::new("yar1.gv")).unwrap());
        //f1.write_str(rope1.to_graphviz().as_slice());
                
        let rope2 = rope1.split(5);

        //let mut f2 = BufferedWriter::new(File::create(&Path::new("yar2.gv")).unwrap());
        //f2.write_str(rope1.to_graphviz().as_slice());
        //f2.write_str(rope2.to_graphviz().as_slice());
        
        assert!(rope1.is_balanced());
        assert!(rope2.is_balanced());
        assert_eq!("Hello", rope1.to_string().as_slice());
        assert_eq!(" there good people of the world!", rope2.to_string().as_slice());
    }
    
    
    #[test]
    fn split_4() {
        let mut rope1 = Rope::new_from_str("Hello there good people of the world!");
        let rope2 = rope1.split(37);
        
        assert!(rope1.is_balanced());
        assert!(rope2.is_balanced());
        assert_eq!("Hello there good people of the world!", rope1.to_string().as_slice());
        assert_eq!("", rope2.to_string().as_slice());
    }
    
    
    #[test]
    fn split_5() {
        let mut rope1 = Rope::new_from_str("Hello there good people of the world!");
        let rope2 = rope1.split(0);
        
        assert!(rope1.is_balanced());
        assert!(rope2.is_balanced());
        assert_eq!("", rope1.to_string().as_slice());
        assert_eq!("Hello there good people of the world!", rope2.to_string().as_slice());
    }
    
    
    #[test]
    fn append_1() {
        let mut rope1 = Rope::new_from_str("Hello there good p");
        let rope2 = Rope::new_from_str("eople of the world!");
        
        rope1.append(rope2);
        
        assert!(rope1.is_balanced());
        assert_eq!("Hello there good people of the world!", rope1.to_string().as_slice());
    }
    
    
    #[test]
    fn append_2() {
        let mut rope1 = Rope::new_from_str("Hello there good people of the world!");
        let rope2 = Rope::new_from_str("");
        
        rope1.append(rope2);
        
        assert!(rope1.is_balanced());
        assert_eq!("Hello there good people of the world!", rope1.to_string().as_slice());
    }
    
    
    #[test]
    fn append_3() {
        let mut rope1 = Rope::new_from_str("");
        let rope2 = Rope::new_from_str("Hello there good people of the world!");
        
        rope1.append(rope2);
        
        assert!(rope1.is_balanced());
        assert_eq!("Hello there good people of the world!", rope1.to_string().as_slice());
    }
    
    
    #[test]
    fn append_4() {
        let mut rope1 = Rope::new_from_str("1234567890-=qwertyuiop{}asdfghjkl;'zxcvbnm,.Hello World!  Let's make this a long string for kicks and giggles.  Who knows when it will end?  No one!  Well, except for the person writing it.  And... eh... later, the person reading it.  Because they'll get to the end.  And then they'll know.");
        let rope2 = Rope::new_from_str("Z");
        
        rope1.append(rope2);
        
        assert!(rope1.is_balanced());
        assert_eq!(rope1.to_string(), "1234567890-=qwertyuiop{}asdfghjkl;'zxcvbnm,.Hello World!  Let's make this a long string for kicks and giggles.  Who knows when it will end?  No one!  Well, except for the person writing it.  And... eh... later, the person reading it.  Because they'll get to the end.  And then they'll know.Z");
    }
    
    
    #[test]
    fn append_5() {
        let mut rope1 = Rope::new_from_str("Z");
        let rope2 = Rope::new_from_str("1234567890-=qwertyuiop{}asdfghjkl;'zxcvbnm,.Hello World!  Let's make this a long string for kicks and giggles.  Who knows when it will end?  No one!  Well, except for the person writing it.  And... eh... later, the person reading it.  Because they'll get to the end.  And then they'll know.");
        
        rope1.append(rope2);
        
        assert!(rope1.is_balanced());
        assert_eq!(rope1.to_string(), "Z1234567890-=qwertyuiop{}asdfghjkl;'zxcvbnm,.Hello World!  Let's make this a long string for kicks and giggles.  Who knows when it will end?  No one!  Well, except for the person writing it.  And... eh... later, the person reading it.  Because they'll get to the end.  And then they'll know.");
    }
    
    
    #[test]
    fn insert_text() {
        let mut rope = Rope::new();
        
        rope.insert_text_at_grapheme_index("Hello 世界!", 0);
        
        let mut iter = rope.grapheme_iter();
        
        assert!(rope.is_balanced());
        assert!(rope.grapheme_count() == 9);
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
    fn insert_text_in_non_empty_buffer_1() {
        let mut rope = Rope::new_from_str("Hello\n 世界\r\n!");
        
        rope.insert_text_at_grapheme_index("Again ", 0);
        
        let mut iter = rope.grapheme_iter();
        
        assert!(rope.is_balanced());
        assert_eq!(rope.grapheme_count(), 17);
        assert_eq!(rope.line_count(), 3);
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
        let mut rope = Rope::new_from_str("Hello\n 世界\r\n!");
        
        rope.insert_text_at_grapheme_index(" again", 5);
        
        let mut iter = rope.grapheme_iter();
        
        assert!(rope.is_balanced());
        assert_eq!(rope.grapheme_count(), 17);
        assert_eq!(rope.line_count(), 3);
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
        let mut rope = Rope::new_from_str("Hello\n 世界\r\n!");
        
        rope.insert_text_at_grapheme_index("again", 6);
        
        let mut iter = rope.grapheme_iter();
        
        assert!(rope.is_balanced());
        assert_eq!(rope.grapheme_count(), 16);
        assert_eq!(rope.line_count(), 3);
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
        let mut rope = Rope::new_from_str("Hello\n 世界\r\n!");        

        rope.insert_text_at_grapheme_index("again", 11);
        
        let mut iter = rope.grapheme_iter();
        
        assert!(rope.is_balanced());
        assert_eq!(rope.grapheme_count(), 16);
        assert_eq!(rope.line_count(), 3);
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
        let mut rope = Rope::new_from_str("Hello\n 世界\r\n!");
        
        rope.insert_text_at_grapheme_index("again", 2);
        
        let mut iter = rope.grapheme_iter();
        
        assert!(rope.is_balanced());
        assert_eq!(rope.grapheme_count(), 16);
        assert_eq!(rope.line_count(), 3);
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
        let mut rope = Rope::new_from_str("Hello\n 世界\r\n!");
        
        rope.insert_text_at_grapheme_index("again", 8);
        
        let mut iter = rope.grapheme_iter();
        
        assert!(rope.is_balanced());
        assert_eq!(rope.grapheme_count(), 16);
        assert_eq!(rope.line_count(), 3);
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
        let mut rope = Rope::new_from_str("Hello\n 世界\r\n!");
        
        rope.insert_text_at_grapheme_index("\nag\n\nain\n", 2);
        
        let mut iter = rope.grapheme_iter();
        
        assert!(rope.is_balanced());
        assert_eq!(rope.grapheme_count(), 20);
        assert_eq!(rope.line_count(), 7);
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
        let mut rope = Rope::new_from_str("Hi\nthere\npeople\nof\nthe\nworld!");
        
        rope.remove_text_between_grapheme_indices(0, 3);
        
        let mut iter = rope.grapheme_iter();
        
        assert!(rope.is_balanced());
        assert_eq!(rope.grapheme_count(), 26);
        assert_eq!(rope.line_count(), 5);
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
        let mut rope = Rope::new_from_str("Hi\nthere\npeople\nof\nthe\nworld!");
        
        rope.remove_text_between_grapheme_indices(0, 12);
        
        let mut iter = rope.grapheme_iter();
        
        assert!(rope.is_balanced());
        assert_eq!(rope.grapheme_count(), 17);
        assert_eq!(rope.line_count(), 4);
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
        let mut rope = Rope::new_from_str("Hi\nthere\npeople\nof\nthe\nworld!");
        
        rope.remove_text_between_grapheme_indices(5, 17);
        
        let mut iter = rope.grapheme_iter();
        
        assert!(rope.is_balanced());
        assert_eq!(rope.grapheme_count(), 17);
        assert_eq!(rope.line_count(), 4);
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
        let mut rope = Rope::new_from_str("Hi\nthere\npeople\nof\nthe\nworld!");
        
        rope.remove_text_between_grapheme_indices(23, 29);
        
        let mut iter = rope.grapheme_iter();
        
        assert!(rope.is_balanced());
        assert_eq!(rope.grapheme_count(), 23);
        assert_eq!(rope.line_count(), 6);
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
        let mut rope = Rope::new_from_str("Hi\nthere\npeople\nof\nthe\nworld!");
        
        rope.remove_text_between_grapheme_indices(17, 29);
        
        let mut iter = rope.grapheme_iter();
        
        assert!(rope.is_balanced());
        assert_eq!(rope.grapheme_count(), 17);
        assert_eq!(rope.line_count(), 4);
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
        let mut rope = Rope::new_from_str("Hello\nworld!");
        
        rope.remove_text_between_grapheme_indices(3, 12);
        
        let mut iter = rope.grapheme_iter();
        
        assert!(rope.is_balanced());
        assert_eq!(rope.grapheme_count(), 3);
        assert_eq!(rope.line_count(), 1);
        assert!(Some("H") == iter.next());
        assert!(Some("e") == iter.next());
        assert!(Some("l") == iter.next());
        assert!(None == iter.next());
    }
    
    
    #[test]
    fn remove_text_7() {
        let mut rope = Rope::new_from_str("Hi\nthere\nworld!");
        
        rope.remove_text_between_grapheme_indices(5, 15);
        
        let mut iter = rope.grapheme_iter();
        
        assert!(rope.is_balanced());
        assert_eq!(rope.grapheme_count(), 5);
        assert_eq!(rope.line_count(), 2);
        assert!(Some("H") == iter.next());
        assert!(Some("i") == iter.next());
        assert!(Some("\n") == iter.next());
        assert!(Some("t") == iter.next());
        assert!(Some("h") == iter.next());
        assert!(None == iter.next());
    }
    
    
    #[test]
    fn remove_text_8() {
        let mut rope = Rope::new_from_str("Hello\nworld!");
        
        rope.remove_text_between_grapheme_indices(3, 11);
        
        let mut iter = rope.grapheme_iter();
        
        assert!(rope.is_balanced());
        assert_eq!(rope.grapheme_count(), 4);
        assert_eq!(rope.line_count(), 1);
        assert!(Some("H") == iter.next());
        assert!(Some("e") == iter.next());
        assert!(Some("l") == iter.next());
        assert!(Some("!") == iter.next());
        assert!(None == iter.next());
    }
    
    
    #[test]
    fn remove_text_9() {
        let mut rope = Rope::new_from_str("Hello\nworld!");
        
        rope.remove_text_between_grapheme_indices(8, 12);
        
        let mut iter = rope.grapheme_iter();
        
        assert!(rope.is_balanced());
        assert_eq!(rope.grapheme_count(), 8);
        assert_eq!(rope.line_count(), 2);
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
        let mut rope = Rope::new_from_str("12\n34\n56\n78");
        
        rope.remove_text_between_grapheme_indices(4, 11);
        
        let mut iter = rope.grapheme_iter();
        
        assert!(rope.is_balanced());
        assert_eq!(rope.grapheme_count(), 4);
        assert_eq!(rope.line_count(), 2);
        assert!(Some("1") == iter.next());
        assert!(Some("2") == iter.next());
        assert!(Some("\n") == iter.next());
        assert!(Some("3") == iter.next());
        assert!(None == iter.next());
    }
    
    
    #[test]
    fn remove_text_11() {
        let mut rope = Rope::new_from_str("1234567890");
        
        rope.remove_text_between_grapheme_indices(9, 10);
        
        let mut iter = rope.grapheme_iter();
        
        assert!(rope.is_balanced());
        assert_eq!(rope.grapheme_count(), 9);
        assert_eq!(rope.line_count(), 1);
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
    fn rebalance_1() {
        let left = Rope::new_from_str(String::from_utf8(vec!['c' as u8; MAX_NODE_SIZE * 64]).unwrap().as_slice());
        let right = Rope::new_from_str(String::from_utf8(vec!['c' as u8; MAX_NODE_SIZE * 1]).unwrap().as_slice());
        
        let mut rope = Rope {
            data: RopeData::Branch(Box::new(left), Box::new(right)),
            grapheme_count_: 0,
            line_ending_count: 0,
            tree_height: 1,
        };
        rope.update_stats();
        
        //let mut f1 = BufferedWriter::new(File::create(&Path::new("yar1.gv")).unwrap());
        //f1.write_str(rope.to_graphviz().as_slice());
        
        rope.rebalance();
        
        //let mut f2 = BufferedWriter::new(File::create(&Path::new("yar2.gv")).unwrap());
        //f2.write_str(rope.to_graphviz().as_slice());
        
        assert!(rope.is_balanced());
    }
    
    
    #[test]
    fn rebalance_2() {
        let left = Rope::new_from_str(String::from_utf8(vec!['c' as u8; MAX_NODE_SIZE * 1]).unwrap().as_slice());
        let right = Rope::new_from_str(String::from_utf8(vec!['c' as u8; MAX_NODE_SIZE * 64]).unwrap().as_slice());
        
        let mut rope = Rope {
            data: RopeData::Branch(Box::new(left), Box::new(right)),
            grapheme_count_: 0,
            line_ending_count: 0,
            tree_height: 1,
        };
        rope.update_stats();
        
        //let mut f1 = BufferedWriter::new(File::create(&Path::new("yar1.gv")).unwrap());
        //f1.write_str(rope.to_graphviz().as_slice());
        
        rope.rebalance();
        
        //let mut f2 = BufferedWriter::new(File::create(&Path::new("yar2.gv")).unwrap());
        //f2.write_str(rope.to_graphviz().as_slice());
        
        assert!(rope.is_balanced());
    }
    
    
    #[test]
    fn rebalance_3() {
        let left = Rope::new_from_str(String::from_utf8(vec!['c' as u8; MAX_NODE_SIZE * 53]).unwrap().as_slice());
        let right = Rope::new_from_str(String::from_utf8(vec!['c' as u8; MAX_NODE_SIZE * 1]).unwrap().as_slice());
        
        let mut rope = Rope {
            data: RopeData::Branch(Box::new(left), Box::new(right)),
            grapheme_count_: 0,
            line_ending_count: 0,
            tree_height: 1,
        };
        rope.update_stats();
        
        //let mut f1 = BufferedWriter::new(File::create(&Path::new("yar1.gv")).unwrap());
        //f1.write_str(rope.to_graphviz().as_slice());
        
        rope.rebalance();
        
        //let mut f2 = BufferedWriter::new(File::create(&Path::new("yar2.gv")).unwrap());
        //f2.write_str(rope.to_graphviz().as_slice());
        
        assert!(rope.is_balanced());
    }
    
    
    #[test]
    fn rebalance_4() {
        let left = Rope::new_from_str(String::from_utf8(vec!['c' as u8; MAX_NODE_SIZE * 1]).unwrap().as_slice());
        let right = Rope::new_from_str(String::from_utf8(vec!['c' as u8; MAX_NODE_SIZE * 53]).unwrap().as_slice());
        
        let mut rope = Rope {
            data: RopeData::Branch(Box::new(left), Box::new(right)),
            grapheme_count_: 0,
            line_ending_count: 0,
            tree_height: 1,
        };
        rope.update_stats();
        
        //let mut f1 = BufferedWriter::new(File::create(&Path::new("yar1.gv")).unwrap());
        //f1.write_str(rope.to_graphviz().as_slice());
        
        rope.rebalance();
        
        //let mut f2 = BufferedWriter::new(File::create(&Path::new("yar2.gv")).unwrap());
        //f2.write_str(rope.to_graphviz().as_slice());

        assert!(rope.is_balanced());
    }
    
    
}

#[cfg(test)]
mod benches {
    use super::*;
    use test::Bencher;
    
    
    #[bench]
    fn new_from_str_1(b: &mut Bencher) {
        b.iter(|| {
            let _ = Rope::new_from_str("
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
        });
    }
    
    
    #[bench]
    fn new_from_str_2(b: &mut Bencher) {
        b.iter(|| {
            let _ = Rope::new_from_str("
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
        });
    }
    
    
    #[bench]
    fn new_from_str_3(b: &mut Bencher) {
        b.iter(|| {
            let _ = Rope::new_from_str("
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
        });
    }
    
    
    #[bench]
    fn new_from_str_4(b: &mut Bencher) {
        b.iter(|| {
            let _ = Rope::new_from_str("
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
        });
    }
    
    
    #[bench]
    fn insert_text_bench_1(b: &mut Bencher) {
        b.iter(|| {
            let mut rope = Rope::new();
            for _ in 0..200 {
                rope.insert_text_at_grapheme_index("Hi", 0);
            }
        });
    }
    
    
    #[bench]
    fn insert_text_bench_2(b: &mut Bencher) {
        b.iter(|| {
            let mut rope = Rope::new();
            for i in 0..200 {
                rope.insert_text_at_grapheme_index("Hi", i/2);
            }
        });
    }
    
    
    #[bench]
    fn insert_text_bench_3(b: &mut Bencher) {
        b.iter(|| {
            let mut rope = Rope::new();
            for i in 0..200 {
                rope.insert_text_at_grapheme_index("Hi", i);
            }
        });
    }
    
    
    #[bench]
    fn insert_large_text_bench_1(b: &mut Bencher) {
        let s = String::from_utf8(vec!['c' as u8; 3457]).unwrap();
        b.iter(|| {
            let mut rope = Rope::new_from_str("Hello there!");
            rope.insert_text_at_grapheme_index(s.as_slice(), 0);
        });
    }
    
    
    #[bench]
    fn insert_large_text_bench_2(b: &mut Bencher) {
        let s = String::from_utf8(vec!['c' as u8; 3457]).unwrap();
        b.iter(|| {
            let mut rope = Rope::new_from_str("Hello there!");
            rope.insert_text_at_grapheme_index(s.as_slice(), 3);
        });
    }
    
    
    #[bench]
    fn insert_large_text_bench_3(b: &mut Bencher) {
        let s = String::from_utf8(vec!['c' as u8; 3457]).unwrap();
        b.iter(|| {
            let mut rope = Rope::new_from_str("Hello there!");
            rope.insert_text_at_grapheme_index(s.as_slice(), 12);
        });
    }
    
    
    #[bench]
    fn remove_text_bench_1(b: &mut Bencher) {
        b.iter(|| {
            let mut rope = Rope::new_from_str("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
            for _ in 0..200 {
                rope.remove_text_between_grapheme_indices(0, 2);
            }
        });
    }
    
    
    #[bench]
    fn remove_text_bench_2(b: &mut Bencher) {
        b.iter(|| {
            let mut rope = Rope::new_from_str("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
            for i in 0..200 {
                rope.remove_text_between_grapheme_indices((200-i)-1, (200-i)+1);
            }
        });
    }
    
    
    #[bench]
    fn remove_text_bench_3(b: &mut Bencher) {
        b.iter(|| {
            let mut rope = Rope::new_from_str("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
            for i in 0..200 {
                rope.remove_text_between_grapheme_indices(400-(i*2)-2, 400-(i*2));
            }
        });
    }
    
    
    #[bench]
    fn append_1(b: &mut Bencher) {
        b.iter(|| {
            let mut left = Rope::new_from_str(String::from_utf8(vec!['c' as u8; 3617]).unwrap().as_slice());
            let right = Rope::new_from_str(String::from_utf8(vec!['c' as u8; 3617]).unwrap().as_slice());
            left.append(right);
        });
    }
    
    
    #[bench]
    fn append_2(b: &mut Bencher) {
        b.iter(|| {
            let mut left = Rope::new_from_str(String::from_utf8(vec!['c' as u8; 263]).unwrap().as_slice());
            let right = Rope::new_from_str(String::from_utf8(vec!['c' as u8; 3617]).unwrap().as_slice());
            left.append(right);
        });
    }
    
    
    #[bench]
    fn append_3(b: &mut Bencher) {
        b.iter(|| {
            let mut left = Rope::new_from_str(String::from_utf8(vec!['c' as u8; 3617]).unwrap().as_slice());
            let right = Rope::new_from_str(String::from_utf8(vec!['c' as u8; 263]).unwrap().as_slice());
            left.append(right);
        });
    }
    
    
    #[bench]
    fn split_1(b: &mut Bencher) {
        b.iter(|| {
            let mut left = Rope::new_from_str(String::from_utf8(vec!['c' as u8; 7649]).unwrap().as_slice());
            let _ = left.split(3617);
        });
    }
    
    
    #[bench]
    fn split_2(b: &mut Bencher) {
        b.iter(|| {
            let mut left = Rope::new_from_str(String::from_utf8(vec!['c' as u8; 7649]).unwrap().as_slice());
            let _ = left.split(263);
        });
    }
}