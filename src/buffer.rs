use std::mem;

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


// /// A rope text storage buffer, using TextBlocks for its underlying text
// /// storage.
// pub enum TextBuffer {
//     Block(TextBlock),
//     Node(Box<TextBuffer>, Box<TextBuffer>)
// }
// 
// impl TextBuffer {
//     pub fn new() -> TextBuffer {
//         TextBuffer::Block(TextBlock::new())
//     }
// }