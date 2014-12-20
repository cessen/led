#![allow(dead_code)]

use std::mem;
use std::fmt;
use super::utils::char_pos_to_byte_pos;

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