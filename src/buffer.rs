use std::cmp::min;
use std::mem;
use std::fmt;


fn newline_count(text: &str) -> uint {
    let mut count = 0;
    for c in text.chars() {
        if c == '\n' {
            count += 1;
        }
    }
    return count;
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
    
    /// Insert 'text' at byte position 'pos'.
    /// NOTE: this makes no attempt to preserve utf8 validity, and may
    /// insert text in the middle of multi-byte code points.
    // TODO: Do checks to prevent invalidating utf8 sequences
    pub fn insert_text(&mut self, text: &str, pos: uint) {
        // Bounds check
        if pos > self.data.len() {
            panic!("TextBlock::insert_text(): attempt to insert text beyond end of text block.");
        }

        // Grow data size		
        self.data.grow(text.len(), 0);
        
        // Move old bytes forward
        let mut from = self.data.len() - text.len();
        let mut to = self.data.len();
        while from > pos {
            from -= 1;
            to -= 1;
            
            self.data[to] = self.data[from];
        }
        
        // Copy new bytes in
        let mut i = pos;
        for b in text.bytes() {
            self.data[i] = b;
            i += 1
        }
    }
    
    /// Remove the text between byte positions 'pos_a' and 'pos_b'.
    /// NOTE: this makes no attempt to preserve utf8 validity, and may
    /// remove parts of multi-byte code points.
    // TODO: Do checks to prevent invalidating utf8 sequences
    pub fn remove_text(&mut self, pos_a: uint, pos_b: uint) {
        // Bounds checks
        if pos_a > pos_b {
            panic!("TextBlock::remove_text(): pos_a must be less than or equal to pos_b.");
        }
        if pos_b > self.data.len() {
            panic!("TextBlock::remove_text(): attempt to remove text beyond the end of text block.");
        }
        
        // Move bytes to fill in the gap left by the removed bytes
        let mut from = pos_b;
        let mut to = pos_a;
        while from < self.data.len() {
            self.data[to] = self.data[from];
            
            from += 1;
            to += 1;
        }
        
        // Remove data from the end
        let final_size = self.data.len() + pos_a - pos_b;
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
    pub newline_count: uint,
    pub byte_count: uint,
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
            newline_count: 0,
            byte_count: 0
        }
    }
    
    pub fn new_from_str(text: &str) -> TextNode {
        TextNode {
            data: TextNodeData::Leaf(TextBlock::new_from_str(text)),
            newline_count: newline_count(text),
            byte_count: text.len()
        }
    }

    /// Splits a leaf node into two roughly equal-sized children
    pub fn split(&mut self) {
        if let TextNodeData::Branch(_, _) = self.data {
            panic!("TextNode::split(): attempt to split a non-leaf node.");
        }
        
        if self.byte_count > 1 {
            // Split data into two new text blocks
            let mut tn1 = box TextNode::new();
            let mut tn2 = box TextNode::new();
            if let TextNodeData::Leaf(ref mut tb) = self.data {
                let pos = tb.len() / 2;
                tn1 = box TextNode::new_from_str(tb.as_str().slice(0, pos));
                tn2 = box TextNode::new_from_str(tb.as_str().slice(pos, tb.len()));
            }
            
            // Swap the old and new data
            let mut new_data = TextNodeData::Branch(tn1, tn2);
            mem::swap(&mut self.data, &mut new_data);
        }
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
        }
    }
    
    /// Insert 'text' at position 'pos'.
    pub fn insert_text(&mut self, text: &str, pos: uint) {
        if pos > self.byte_count {
            panic!("TextNode::insert_text(): attempt to insert text after end of node text.");
        }
        
        match self.data {
            TextNodeData::Leaf(_) => {
                if let TextNodeData::Leaf(ref mut tb) = self.data {
                    tb.insert_text(text, pos);
                    
                    self.newline_count += newline_count(text);
                    self.byte_count = tb.len();
                }
                
                if self.byte_count > MAX_LEAF_SIZE {
                    self.split();
                }
            },
            
            TextNodeData::Branch(ref mut left, ref mut right) => {
                if pos <= left.byte_count {
                    left.insert_text(text, pos);
                }
                else {
                    right.insert_text(text, pos - left.byte_count);
                }
                
                self.newline_count = left.newline_count + right.newline_count;
                self.byte_count = left.byte_count + right.byte_count;
            }
        }
    }
    
    /// Remove the text between byte positions 'pos_a' and 'pos_b'.
    pub fn remove_text(&mut self, pos_a: uint, pos_b: uint) {
        // Bounds checks
        if pos_a > pos_b {
            panic!("TextNode::remove_text(): pos_a must be less than or equal to pos_b.");
        }
        if pos_b > self.byte_count {
            panic!("TextNode::remove_text(): attempt to remove text after end of node text.");
        }
        
        match self.data {
            TextNodeData::Leaf(ref mut tb) => {
                tb.remove_text(pos_a, pos_b);
                
                self.newline_count = newline_count(tb.as_str());
                self.byte_count = tb.len();
            },
            
            TextNodeData::Branch(ref mut left, ref mut right) => {
                let lbc = left.byte_count;
                
                if pos_a < lbc {
                    left.remove_text(pos_a, min(pos_b, lbc));
                }
                
                if pos_b > lbc {
                    right.remove_text(pos_a - min(pos_a, lbc), pos_b - lbc);
                }
                
                self.newline_count = left.newline_count + right.newline_count;
                self.byte_count = left.byte_count + right.byte_count;
            }
        }
        
        if self.byte_count < MIN_LEAF_SIZE {
            self.merge();
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
    root: TextNode
}

impl TextBuffer {
    pub fn new() -> TextBuffer {
        TextBuffer {
            root: TextNode::new()
        }
    }
    
    pub fn len(&self) -> uint {
        self.root.byte_count
    }
    
    /// Insert 'text' at byte position 'pos'.
    pub fn insert_text(&mut self, text: &str, pos: uint) {
        self.root.insert_text(text, pos);
    }
    
    /// Remove the text between byte positions 'pos_a' and 'pos_b'.
    pub fn remove_text(&mut self, pos_a: uint, pos_b: uint) {
        self.root.remove_text(pos_a, pos_b);
    }
}

impl fmt::Show for TextBuffer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.root.fmt(f)
    }
}