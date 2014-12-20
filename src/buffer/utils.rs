#![allow(dead_code)]
//! Misc helpful utility functions for TextBuffer related stuff.

pub fn newline_count(text: &str) -> uint {
    let mut count = 0;
    for c in text.chars() {
        if c == '\n' {
            count += 1;
        }
    }
    return count;
}

pub fn char_count(text: &str) -> uint {
    let mut count = 0;
    for _ in text.chars() {
        count += 1;
    }
    return count;
}

pub fn char_and_newline_count(text: &str) -> (uint, uint) {
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

pub fn char_pos_to_byte_pos(text: &str, pos: uint) -> uint {
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