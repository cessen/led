#![allow(dead_code)]

use std::mem;
use std::fmt;
use string_utils::{char_pos_to_byte_pos, char_count};

/// A block of text, contiguous in memory
pub struct TextBlock {
    // The actual text data, in utf8
    pub data: Vec<u8>,
    
    // The visual width of each printable character.
    // Characters with variable width (e.g. tab characters)
    // have width None.
    pub widths: Vec<Option<u8>>,
}

impl TextBlock {
    /// Create a new empty text block.
    pub fn new() -> TextBlock {
        TextBlock {
            data: Vec::<u8>::new(),
            widths: Vec::<Option<u8>>::new(),
        }
    }
    
    /// Create a new text block with the contents of 'text'.
    pub fn new_from_str(text: &str) -> TextBlock {
        let mut tb = TextBlock {
            data: Vec::<u8>::with_capacity(text.len()),
            widths: Vec::<Option<u8>>::with_capacity(text.len()),
        };
        
        for b in text.bytes() {
            tb.data.push(b);
        }
        
        // TODO: handle fonts
        for c in text.chars() {
            if c == '\t' {
                tb.widths.push(None);
            }
            else {
                tb.widths.push(Some(1));
            }
        }
        
        return tb;
    }
    
    /// Return the length of the text block in bytes.
    pub fn len(&self) -> uint {
        self.data.len()
    }
    
    /// Returns the total width of text block sans-variable-width characters
    pub fn total_non_variable_width(&self) -> uint {
        let mut width: uint = 0;
        
        for w in self.widths.iter() {
            if let &Some(ww) = w {
                width += ww as uint;
            }
        }
        
        return width;
    }
    
    /// Returns the number of variable-width chars in the text block
    pub fn variable_width_chars(&self) -> uint {
        let mut count: uint = 0;
        
        for w in self.widths.iter() {
            if let &None = w {
                count += 1;
            }
        }
        
        return count;
    }
    
    /// Insert 'text' at char position 'pos'.
    pub fn insert_text(&mut self, text: &str, pos: uint) {
        //====== TEXT DATA ======
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
        
        //====== WIDTHS ======
        // Grow widths size
        let cc = char_count(text);
        self.widths.grow(cc, None);
        
        // Move old widths forward
        from = self.widths.len() - cc;
        to = self.widths.len();
        while from > pos {
            from -= 1;
            to -= 1;
            
            self.widths[to] = self.widths[from];
        }
        
        // Copy new widths in
        i = pos;
        for c in text.chars() {
            if c == '\t' {
                self.widths[i] = None;
            }
            else {
                self.widths[i] = Some(1);
            }
            i += 1
        }
    }
    
    /// Remove the text between char positions 'pos_a' and 'pos_b'.
    pub fn remove_text(&mut self, pos_a: uint, pos_b: uint) {
        // Bounds checks
        if pos_a > pos_b {
            panic!("TextBlock::remove_text(): pos_a must be less than or equal to pos_b.");
        }
        
        //====== TEXT DATA ======
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
        let final_data_size = self.data.len() + byte_pos_a - byte_pos_b;
        self.data.truncate(final_data_size);
        
        //====== WIDTHS ======
        from = pos_b;
        to = pos_a;
        while from < self.widths.len() {
            self.widths[to] = self.widths[from];
            
            from += 1;
            to += 1;
        }
        
        // Remove data from end
        let final_widths_size = self.widths.len() + pos_a - pos_b;
        self.data.truncate(final_widths_size);
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