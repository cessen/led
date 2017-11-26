#![allow(dead_code)]
//! Misc helpful utility functions for TextBuffer related stuff.

use std::str::CharIndices;
use std::iter::repeat;
use unicode_segmentation::UnicodeSegmentation;


pub fn is_line_ending(text: &str) -> bool {
    match text {
        "\u{000D}\u{000A}"
        | "\u{000A}"
        | "\u{000B}"
        | "\u{000C}"
        | "\u{000D}"
        | "\u{0085}"
        | "\u{2028}"
        | "\u{2029}" => true,
        
        _ => false
    }
}

pub fn line_ending_count(text: &str) -> usize {
    let mut count = 0;
    for g in UnicodeSegmentation::graphemes(text, true) {
        if is_line_ending(g) {
            count += 1;
        }
    }
    return count;
}

pub fn char_count(text: &str) -> usize {
    let mut count = 0;
    for _ in text.chars() {
        count += 1;
    }
    return count;
}

pub fn grapheme_count(text: &str) -> usize {
    let mut count = 0;
    for _ in UnicodeSegmentation::graphemes(text, true) {
        count += 1;
    }
    return count;
}

pub fn grapheme_count_is_less_than(text: &str, n: usize) -> bool {
    let mut count = 0;
    for _ in UnicodeSegmentation::graphemes(text, true) {
        count += 1;
        if count >= n {
            return false;
        }
    }
    
    return true;
}

pub fn char_grapheme_line_ending_count(text: &str) -> (usize, usize, usize) {
    let mut cc = 0;
    let mut gc = 0;
    let mut lec = 0;
    
    for g in UnicodeSegmentation::graphemes(text, true) {
        cc += char_count(g);
        gc += 1;
        if is_line_ending(g) {
            lec += 1;
        }
    }
    
    return (cc, gc, lec);
}

pub fn graphemes_are_mergeable(s1: &str, s2: &str) -> bool {
    let mut s = String::with_capacity(s1.len() + s2.len());
    s.push_str(s1);
    s.push_str(s2);
    
    return grapheme_count(&s[..]) == 1;
}

pub fn char_pos_to_byte_pos(text: &str, pos: usize) -> usize {
    let mut i: usize = 0;
    
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

pub fn grapheme_pos_to_byte_pos(text: &str, pos: usize) -> usize {
    let mut i: usize = 0;
    
    for (offset, _) in UnicodeSegmentation::grapheme_indices(text, true) {
        if i == pos {
            return offset;
        }
        i += 1;
    }
    
    if i == pos {
        return text.len();
    }
    
    panic!("grapheme_pos_to_byte_pos(): grapheme position off the end of the string.");
}

pub fn char_pos_to_grapheme_pos(text: &str, pos: usize) -> usize {
    let mut i = 0usize;
    let mut cc = 0usize;
    
    for g in UnicodeSegmentation::graphemes(text, true) {
        if cc == pos {
            return i;
        }
        
        cc += char_count(g);
        
        if cc > pos {
            return i;
        }
        
        i += 1;
    }
    
    if cc == pos {
        return i;
    }
    
    panic!("char_pos_to_grapheme_pos(): char position off the end of the string.");
}

pub fn grapheme_pos_to_char_pos(text: &str, pos: usize) -> usize {
    let mut i = 0usize;
    let mut cc = 0usize;
    
    for g in UnicodeSegmentation::graphemes(text, true) {
        if i == pos {
            return cc;
        }
        
        cc += char_count(g);
        i += 1;
    }
    
    if i == pos {
        return cc;
    }
    
    panic!("grapheme_pos_to_char_pos(): grapheme position off the end of the string.");
}

/// Inserts the given text into the given string at the given grapheme index.
pub fn insert_text_at_char_index(s: &mut String, text: &str, pos: usize) {
    // Find insertion position in bytes
    let byte_pos = char_pos_to_byte_pos(&s[..], pos);
    
    // Get byte vec of string
    let byte_vec = unsafe { s.as_mut_vec() };
    
    // Grow data size        
    byte_vec.extend(repeat(0).take(text.len()));
    
    // Move old bytes forward
    // TODO: use copy_memory()...?
    let mut from = byte_vec.len() - text.len();
    let mut to = byte_vec.len();
    while from > byte_pos {
        from -= 1;
        to -= 1;
        
        byte_vec[to] = byte_vec[from];
    }
    
    // Copy new bytes in
    // TODO: use copy_memory()
    let mut i = byte_pos;
    for b in text.bytes() {
        byte_vec[i] = b;
        i += 1
    }
}

/// Inserts the given text into the given string at the given grapheme index.
pub fn insert_text_at_grapheme_index(s: &mut String, text: &str, pos: usize) {
    // Find insertion position in bytes
    let byte_pos = grapheme_pos_to_byte_pos(&s[..], pos);
    
    // Get byte vec of string
    let byte_vec = unsafe { s.as_mut_vec() };
    
    // Grow data size        
    byte_vec.extend(repeat(0).take(text.len()));
    
    // Move old bytes forward
    // TODO: use copy_memory()...?
    let mut from = byte_vec.len() - text.len();
    let mut to = byte_vec.len();
    while from > byte_pos {
        from -= 1;
        to -= 1;
        
        byte_vec[to] = byte_vec[from];
    }
    
    // Copy new bytes in
    // TODO: use copy_memory()
    let mut i = byte_pos;
    for g in UnicodeSegmentation::graphemes(text, true) {
        
        for b in g.bytes() {
            byte_vec[i] = b;
            i += 1
        }
    }
}

/// Removes the text between the given char indices in the given string.
pub fn remove_text_between_char_indices(s: &mut String, pos_a: usize, pos_b: usize) {
    // Bounds checks
    assert!(pos_a <= pos_b, "remove_text_between_char_indices(): pos_a must be less than or equal to pos_b.");
    
    if pos_a == pos_b {
        return;
    }
    
    // Find removal positions in bytes
    // TODO: get both of these in a single pass
    let byte_pos_a = char_pos_to_byte_pos(&s[..], pos_a);
    let byte_pos_b = char_pos_to_byte_pos(&s[..], pos_b);
    
    // Get byte vec of string
    let byte_vec = unsafe { s.as_mut_vec() };
    
    // Move bytes to fill in the gap left by the removed bytes
    let mut from = byte_pos_b;
    let mut to = byte_pos_a;
    while from < byte_vec.len() {
        byte_vec[to] = byte_vec[from];
        
        from += 1;
        to += 1;
    }
    
    // Remove data from the end
    let final_text_size = byte_vec.len() + byte_pos_a - byte_pos_b;
    byte_vec.truncate(final_text_size);
}

/// Removes the text between the given grapheme indices in the given string.
pub fn remove_text_between_grapheme_indices(s: &mut String, pos_a: usize, pos_b: usize) {
    // Bounds checks
    assert!(pos_a <= pos_b, "remove_text_between_grapheme_indices(): pos_a must be less than or equal to pos_b.");
    
    if pos_a == pos_b {
        return;
    }
    
    // Find removal positions in bytes
    // TODO: get both of these in a single pass
    let byte_pos_a = grapheme_pos_to_byte_pos(&s[..], pos_a);
    let byte_pos_b = grapheme_pos_to_byte_pos(&s[..], pos_b);
    
    // Get byte vec of string
    let byte_vec = unsafe { s.as_mut_vec() };
    
    // Move bytes to fill in the gap left by the removed bytes
    let mut from = byte_pos_b;
    let mut to = byte_pos_a;
    while from < byte_vec.len() {
        byte_vec[to] = byte_vec[from];
        
        from += 1;
        to += 1;
    }
    
    // Remove data from the end
    let final_text_size = byte_vec.len() + byte_pos_a - byte_pos_b;
    byte_vec.truncate(final_text_size);
}

/// Splits a string into two strings at the char index given.
/// The first section of the split is stored in the original string,
/// while the second section of the split is returned as a new string.
pub fn split_string_at_char_index(s1: &mut String, pos: usize) -> String {
    let mut s2 = String::new();
    
    // Code block to contain the borrow of s2
    {
        let byte_pos = char_pos_to_byte_pos(&s1[..], pos);
        
        let byte_vec_1 = unsafe { s1.as_mut_vec() };
        let byte_vec_2 = unsafe { s2.as_mut_vec() };
        
        byte_vec_2.extend((&byte_vec_1[byte_pos..]).iter().map(|&i| i));
        byte_vec_1.truncate(byte_pos);
    }
    
    return s2;
}

/// Splits a string into two strings at the grapheme index given.
/// The first section of the split is stored in the original string,
/// while the second section of the split is returned as a new string.
pub fn split_string_at_grapheme_index(s1: &mut String, pos: usize) -> String {
    let mut s2 = String::new();
    
    // Code block to contain the borrow of s2
    {
        let byte_pos = grapheme_pos_to_byte_pos(&s1[..], pos);
        
        let byte_vec_1 = unsafe { s1.as_mut_vec() };
        let byte_vec_2 = unsafe { s2.as_mut_vec() };
        
        byte_vec_2.extend((&byte_vec_1[byte_pos..]).iter().map(|&i| i));
        byte_vec_1.truncate(byte_pos);
    }
    
    return s2;
}


/// A grapheme iterator that only recognizes CRLF as a composite grapheme.
/// This is only temporary, a stand-in for the proper Graphemes iterator
/// from stdlib which is currently marked unstable and thus is unavailable
/// in stable Rust builds.  When Graphemes makes its way back into stdlib
/// or is split into another library, replace this with it.
pub struct TempGraphemeIndices<'a> {
    s: &'a str,
    chars: CharIndices<'a>,
    i1: usize,
    i2: usize,
}

impl<'a> TempGraphemeIndices<'a> {
    fn push_i2(&mut self) {
        if let Some((i, _)) = self.chars.next() {
            self.i2 = i;
        }
        else {
            self.i2 = self.s.len();
        }
    }
}

impl<'a> Iterator for TempGraphemeIndices<'a> {
    type Item = (usize, &'a str);
    
    fn next(&mut self) -> Option<(usize, &'a str)> {
        // Advance the iterator
        if self.i1 == self.i2 {
            self.push_i2();
        }
    
        // If we're at the end, we're done
        if self.i1 == self.s.len() {
            return None;
        }
    
        
        match &(self.s)[self.i1 .. self.i2] {
            "\u{000D}" => {
                let ii = self.i1;
                let i3 = self.i2;
                self.push_i2();
                if &(self.s)[self.i1 .. self.i2] == "\u{000D}\u{000A}" {
                    let s = &(self.s)[self.i1 .. self.i2];
                    self.i1 = self.i2;
                    return Some((ii, s));
                }
                else {
                    let ii = self.i1;
                    let s = &(self.s)[self.i1 .. i3];
                    self.i1 = i3;
                    return Some((ii, s));
                }
            },
            
            _ => {
                let ii = self.i1;
                let s = &(self.s)[self.i1 .. self.i2];
                self.i1 = self.i2;
                return Some((ii, s));
            },
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn graphemes_are_mergeable_1() {
        assert!(graphemes_are_mergeable("\u{000D}", "\u{000A}"));
        assert!(!graphemes_are_mergeable("\u{000A}", "\u{000D}"));
        assert!(!graphemes_are_mergeable("a", "b"));
    }
    
    #[test]
    fn char_pos_to_grapheme_pos_1() {
        let s = "Hello\u{000D}\u{000A}there!";
        
        assert_eq!(char_pos_to_grapheme_pos(s, 0), 0);
        assert_eq!(char_pos_to_grapheme_pos(s, 5), 5);
        assert_eq!(char_pos_to_grapheme_pos(s, 6), 5);
        assert_eq!(char_pos_to_grapheme_pos(s, 7), 6);
        assert_eq!(char_pos_to_grapheme_pos(s, 13), 12);
    }
    
    #[test]
    fn char_pos_to_grapheme_pos_2() {
        let s = "a";
        
        assert_eq!(char_pos_to_grapheme_pos(s, 0), 0);
        assert_eq!(char_pos_to_grapheme_pos(s, 1), 1);
    }
    
    #[test]
    fn char_pos_to_grapheme_pos_3() {
        let s = "\u{000D}\u{000A}";
        
        assert_eq!(char_pos_to_grapheme_pos(s, 0), 0);
        assert_eq!(char_pos_to_grapheme_pos(s, 1), 0);
        assert_eq!(char_pos_to_grapheme_pos(s, 2), 1);
    }
    
    #[test]
    fn char_pos_to_grapheme_pos_4() {
        let s = "";
        
        assert_eq!(char_pos_to_grapheme_pos(s, 0), 0);
    }
    
    #[test]
    fn grapheme_pos_to_char_pos_1() {
        let s = "Hello\u{000D}\u{000A}there!";
        
        assert_eq!(grapheme_pos_to_char_pos(s, 0), 0);
        assert_eq!(grapheme_pos_to_char_pos(s, 5), 5);
        assert_eq!(grapheme_pos_to_char_pos(s, 6), 7);
        assert_eq!(grapheme_pos_to_char_pos(s, 8), 9);
        assert_eq!(grapheme_pos_to_char_pos(s, 12), 13);
    }
    
    #[test]
    fn grapheme_pos_to_char_pos_2() {
        let s = "a";
        
        assert_eq!(grapheme_pos_to_char_pos(s, 0), 0);
        assert_eq!(grapheme_pos_to_char_pos(s, 1), 1);
    }
    
    #[test]
    fn grapheme_pos_to_char_pos_3() {
        let s = "\u{000D}\u{000A}";
        
        assert_eq!(grapheme_pos_to_char_pos(s, 0), 0);
        assert_eq!(grapheme_pos_to_char_pos(s, 1), 2);
    }
    
    #[test]
    fn grapheme_pos_to_char_pos_4() {
        let s = "";
        
        assert_eq!(grapheme_pos_to_char_pos(s, 0), 0);
    }
}