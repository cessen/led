#![cfg(test)]
#![allow(unused_imports)]

use std::iter;
use string_utils::{remove_text_between_char_indices};
use super::{Rope, RopeData, RopeGraphemeIter, MAX_NODE_SIZE};
//use std::old_path::Path;
//use std::old_io::fs::File;
//use std::old_io::BufferedWriter;


#[test]
fn new_1() {
    let rope = Rope::new();
    let mut iter = rope.grapheme_iter();
    
    assert_eq!(rope.char_count(), 0);
    assert_eq!(rope.grapheme_count(), 0);
    assert_eq!(rope.line_ending_count(), 0);
    
    assert_eq!(None, iter.next());
}


#[test]
fn new_2() {
    let rope = Rope::from_str("Hello world!");
    let mut iter = rope.grapheme_iter();
    
    assert_eq!(rope.char_count(), 12);
    assert_eq!(rope.grapheme_count(), 12);
    assert_eq!(rope.line_ending_count(), 0);
    
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
    let s = "Hello world!".to_string();
    let rope = Rope::from_string(s);
    let mut iter = rope.grapheme_iter();
    
    assert_eq!(rope.char_count(), 12);
    assert_eq!(rope.grapheme_count(), 12);
    assert_eq!(rope.line_ending_count(), 0);
    
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
    let rope = Rope::from_str(&(String::from_utf8(vec!['c' as u8; 1 + MAX_NODE_SIZE * 53]).unwrap())[..]);
    
    assert_eq!(rope.char_count(), 1 + MAX_NODE_SIZE * 53);
    assert_eq!(rope.grapheme_count(), 1 + MAX_NODE_SIZE * 53);
    assert_eq!(rope.line_ending_count(), 0);
    
    assert!(rope.is_balanced());
}


#[test]
fn counts() {
    let rope = Rope::from_str("Hello\u{000D}\u{000A}world!");
    
    assert_eq!(rope.char_count(), 13);
    assert_eq!(rope.grapheme_count(), 12);
    assert_eq!(rope.line_ending_count(), 1);
    assert_eq!(rope.grapheme_count_in_char_range(0, 13), 12);
}


#[test]
fn grapheme_count_in_char_range() {
    let rope = Rope::from_str("Hello\u{000D}\u{000A}world!");
    
    assert_eq!(rope.grapheme_count_in_char_range(5, 13), 7);
    assert_eq!(rope.grapheme_count_in_char_range(6, 13), 7);
    assert_eq!(rope.grapheme_count_in_char_range(7, 13), 6);
    
    assert_eq!(rope.grapheme_count_in_char_range(0, 7), 6);
    assert_eq!(rope.grapheme_count_in_char_range(0, 6), 6);
    assert_eq!(rope.grapheme_count_in_char_range(0, 5), 5);
    
    assert_eq!(rope.grapheme_count_in_char_range(5, 7), 1);
    assert_eq!(rope.grapheme_count_in_char_range(5, 6), 1);
    assert_eq!(rope.grapheme_count_in_char_range(6, 7), 1);
}


#[test]
fn char_at_index() {
    let rope = Rope::from_str("Hel世界lo\u{000D}\u{000A}world!");
    
    assert_eq!(rope.char_count(), 15);
    assert_eq!(rope.grapheme_count(), 14);
    assert_eq!(rope.line_ending_count(), 1);
    
    assert_eq!('H', rope.char_at_index(0));
    assert_eq!('界', rope.char_at_index(4));
    assert_eq!('\u{000D}', rope.char_at_index(7));
    assert_eq!('\u{000A}', rope.char_at_index(8));
    assert_eq!('w', rope.char_at_index(9));
    assert_eq!('!', rope.char_at_index(14));
}


#[test]
fn grapheme_at_index() {
    let rope = Rope::from_str("Hel世界lo\u{000D}\u{000A}world!");
    
    assert_eq!(rope.char_count(), 15);
    assert_eq!(rope.grapheme_count(), 14);
    assert_eq!(rope.line_ending_count(), 1);
    
    assert_eq!("H", rope.grapheme_at_index(0));
    assert_eq!("界", rope.grapheme_at_index(4));
    assert_eq!("\u{000D}\u{000A}", rope.grapheme_at_index(7));
    assert_eq!("w", rope.grapheme_at_index(8));
    assert_eq!("!", rope.grapheme_at_index(13));
}


#[test]
fn char_iter_1() {
    let rope = Rope::from_str("Hel世界lo\u{000D}\u{000A}world!");
    
    let mut iter = rope.char_iter();
    
    assert!(Some('H') == iter.next());
    assert!(Some('e') == iter.next());
    assert!(Some('l') == iter.next());
    assert!(Some('世') == iter.next());
    assert!(Some('界') == iter.next());
    assert!(Some('l') == iter.next());
    assert!(Some('o') == iter.next());
    assert!(Some('\u{000D}') == iter.next());
    assert!(Some('\u{000A}') == iter.next());
    assert!(Some('w') == iter.next());
    assert!(Some('o') == iter.next());
    assert!(Some('r') == iter.next());
    assert!(Some('l') == iter.next());
    assert!(Some('d') == iter.next());
    assert!(Some('!') == iter.next());
    assert!(None == iter.next());
}


#[test]
fn char_iter_2() {
    let rope = Rope::from_str("Hel世界lo\u{000D}\u{000A}world!");
    
    let mut iter = rope.char_iter_at_index(8);
    
    assert!(Some('\u{000A}') == iter.next());
    assert!(Some('w') == iter.next());
    assert!(Some('o') == iter.next());
    assert!(Some('r') == iter.next());
    assert!(Some('l') == iter.next());
    assert!(Some('d') == iter.next());
    assert!(Some('!') == iter.next());
    assert!(None == iter.next());
}


#[test]
fn char_iter_3() {
    let rope = Rope::from_str("Hel世界lo\u{000D}\u{000A}world!");
    
    let mut iter = rope.char_iter_at_index(9);
    
    assert!(Some('w') == iter.next());
    assert!(Some('o') == iter.next());
    assert!(Some('r') == iter.next());
    assert!(Some('l') == iter.next());
    assert!(Some('d') == iter.next());
    assert!(Some('!') == iter.next());
    assert!(None == iter.next());
}


#[test]
fn char_iter_4() {
    let rope = Rope::from_str("Hel世界lo\u{000D}\u{000A}world!");
    
    let mut iter = rope.char_iter_between_indices(9, 12);
    
    assert!(Some('w') == iter.next());
    assert!(Some('o') == iter.next());
    assert!(Some('r') == iter.next());
    assert!(None == iter.next());
}


#[test]
fn grapheme_iter_1() {
    let rope = Rope::from_str("Hel世界lo\u{000D}\u{000A}world!");
    
    let mut iter = rope.grapheme_iter();
    
    assert_eq!(Some("H"), iter.next());
    assert_eq!(Some("e"), iter.next());
    assert_eq!(Some("l"), iter.next());
    assert_eq!(Some("世"), iter.next());
    assert_eq!(Some("界"), iter.next());
    assert_eq!(Some("l"), iter.next());
    assert_eq!(Some("o"), iter.next());
    assert_eq!(Some("\u{000D}\u{000A}"), iter.next());
    assert_eq!(Some("w"), iter.next());
    assert_eq!(Some("o"), iter.next());
    assert_eq!(Some("r"), iter.next());
    assert_eq!(Some("l"), iter.next());
    assert_eq!(Some("d"), iter.next());
    assert_eq!(Some("!"), iter.next());
    assert_eq!(None, iter.next());
}


#[test]
fn grapheme_iter_2() {
    let rope = Rope::from_str("Hel世界lo\u{000D}\u{000A}world!");
    
    let mut iter = rope.grapheme_iter_at_index(7);
    
    assert_eq!(Some("\u{000D}\u{000A}"), iter.next());
    assert_eq!(Some("w"), iter.next());
    assert_eq!(Some("o"), iter.next());
    assert_eq!(Some("r"), iter.next());
    assert_eq!(Some("l"), iter.next());
    assert_eq!(Some("d"), iter.next());
    assert_eq!(Some("!"), iter.next());
    assert_eq!(None, iter.next());
}


#[test]
fn grapheme_iter_3() {
    let rope = Rope::from_str("Hel世界lo\u{000D}\u{000A}world!");
    
    let mut iter = rope.grapheme_iter_at_index(8);
    
    assert_eq!(Some("w"), iter.next());
    assert_eq!(Some("o"), iter.next());
    assert_eq!(Some("r"), iter.next());
    assert_eq!(Some("l"), iter.next());
    assert_eq!(Some("d"), iter.next());
    assert_eq!(Some("!"), iter.next());
    assert_eq!(None, iter.next());
}


#[test]
fn grapheme_iter_4() {
    let rope = Rope::from_str("Hel世界lo\u{000D}\u{000A}world!");
    
    let mut iter = rope.grapheme_iter_between_indices(8, 11);
    
    assert_eq!(Some("w"), iter.next());
    assert_eq!(Some("o"), iter.next());
    assert_eq!(Some("r"), iter.next());
    assert_eq!(None, iter.next());
}


#[test]
fn slice_1() {
    let rope = Rope::from_str("Hello everyone!  How are you doing, eh?");
    let s = rope.slice(0, 15);
    
    let mut iter = s.char_iter();
    
    assert_eq!(s.char_count(), 15);
    assert_eq!(s.grapheme_count(), 15);
    assert!(Some('H') == iter.next());
    assert!(Some('e') == iter.next());
    assert!(Some('l') == iter.next());
    assert!(Some('l') == iter.next());
    assert!(Some('o') == iter.next());
    assert!(Some(' ') == iter.next());
    assert!(Some('e') == iter.next());
    assert!(Some('v') == iter.next());
    assert!(Some('e') == iter.next());
    assert!(Some('r') == iter.next());
    assert!(Some('y') == iter.next());
    assert!(Some('o') == iter.next());
    assert!(Some('n') == iter.next());
    assert!(Some('e') == iter.next());
    assert!(Some('!') == iter.next());
    assert!(None == iter.next());
}


#[test]
fn slice_2() {
    let rope = Rope::from_str("Hello everyone!  How are you doing, eh?");
    let s = rope.slice(6, 20);
    
    let mut iter = s.char_iter();
    
    assert_eq!(s.char_count(), 14);
    assert_eq!(s.grapheme_count(), 14);
    assert!(Some('e') == iter.next());
    assert!(Some('v') == iter.next());
    assert!(Some('e') == iter.next());
    assert!(Some('r') == iter.next());
    assert!(Some('y') == iter.next());
    assert!(Some('o') == iter.next());
    assert!(Some('n') == iter.next());
    assert!(Some('e') == iter.next());
    assert!(Some('!') == iter.next());
    assert!(Some(' ') == iter.next());
    assert!(Some(' ') == iter.next());
    assert!(Some('H') == iter.next());
    assert!(Some('o') == iter.next());
    assert!(Some('w') == iter.next());
    assert!(None == iter.next());
}


#[test]
fn slice_3() {
    let rope = Rope::from_str("Hello everyone!  How are you doing, eh?");
    let s = rope.slice(21, 39);
    
    let mut iter = s.char_iter();
    
    assert_eq!(s.char_count(), 18);
    assert_eq!(s.grapheme_count(), 18);
    assert!(Some('a') == iter.next());
    assert!(Some('r') == iter.next());
    assert!(Some('e') == iter.next());
    assert!(Some(' ') == iter.next());
    assert!(Some('y') == iter.next());
    assert!(Some('o') == iter.next());
    assert!(Some('u') == iter.next());
    assert!(Some(' ') == iter.next());
    assert!(Some('d') == iter.next());
    assert!(Some('o') == iter.next());
    assert!(Some('i') == iter.next());
    assert!(Some('n') == iter.next());
    assert!(Some('g') == iter.next());
    assert!(Some(',') == iter.next());
    assert!(Some(' ') == iter.next());
    assert!(Some('e') == iter.next());
    assert!(Some('h') == iter.next());
    assert!(Some('?') == iter.next());
    assert!(None == iter.next());
}


#[test]
fn slice_4() {
    let rope = Rope::from_str("Hello everyone!  How are you doing, eh?");
    let s = rope.slice(21, 39);
    
    let mut iter = s.char_iter();
    
    assert_eq!(s.char_count(), 18);
    assert_eq!(s.grapheme_count(), 18);
    assert!(Some('a') == iter.next());
    assert!(Some('r') == iter.next());
    assert!(Some('e') == iter.next());
    assert!(Some(' ') == iter.next());
    assert!(Some('y') == iter.next());
    assert!(Some('o') == iter.next());
    assert!(Some('u') == iter.next());
    assert!(Some(' ') == iter.next());
    assert!(Some('d') == iter.next());
    assert!(Some('o') == iter.next());
    assert!(Some('i') == iter.next());
    assert!(Some('n') == iter.next());
    assert!(Some('g') == iter.next());
    assert!(Some(',') == iter.next());
    assert!(Some(' ') == iter.next());
    assert!(Some('e') == iter.next());
    assert!(Some('h') == iter.next());
    assert!(Some('?') == iter.next());
    assert!(None == iter.next());
}


#[test]
fn slice_5() {
    let rope = Rope::from_str("Hello everyone!  How are you doing, eh?");
    let s = rope.slice(21, 39);
    let s2 = s.slice(3, 10);
    
    let mut iter = s2.char_iter();
    
    assert_eq!(s.char_count(), 18);
    assert_eq!(s.grapheme_count(), 18);
    assert!(Some(' ') == iter.next());
    assert!(Some('y') == iter.next());
    assert!(Some('o') == iter.next());
    assert!(Some('u') == iter.next());
    assert!(Some(' ') == iter.next());
    assert!(Some('d') == iter.next());
    assert!(Some('o') == iter.next());
    assert!(None == iter.next());
}


#[test]
fn slice_6() {
    let rope = Rope::from_str("Hello everyone!  How are you doing, eh?");
    let s = rope.slice(15, 39);
    
    let mut iter = s.char_iter_between_indices(0, 24);
    
    assert!(Some(' ') == iter.next());
    assert!(Some(' ') == iter.next());
    assert!(Some('H') == iter.next());
    assert!(Some('o') == iter.next());
    assert!(Some('w') == iter.next());
    assert!(Some(' ') == iter.next());
    assert!(Some('a') == iter.next());
    assert!(Some('r') == iter.next());
    assert!(Some('e') == iter.next());
    assert!(Some(' ') == iter.next());
    assert!(Some('y') == iter.next());
    assert!(Some('o') == iter.next());
    assert!(Some('u') == iter.next());
    assert!(Some(' ') == iter.next());
    assert!(Some('d') == iter.next());
    assert!(Some('o') == iter.next());
    assert!(Some('i') == iter.next());
    assert!(Some('n') == iter.next());
    assert!(Some('g') == iter.next());
    assert!(Some(',') == iter.next());
    assert!(Some(' ') == iter.next());
    assert!(Some('e') == iter.next());
    assert!(Some('h') == iter.next());
    assert!(Some('?') == iter.next());
    assert!(None == iter.next());
}


#[test]
fn slice_7() {
    let rope = Rope::from_str("Hello everyone!  How are you doing, eh?");
    let s = rope.slice(15, 39);
    
    let mut iter = s.char_iter_between_indices(10, 20);
    
    assert!(Some('y') == iter.next());
    assert!(Some('o') == iter.next());
    assert!(Some('u') == iter.next());
    assert!(Some(' ') == iter.next());
    assert!(Some('d') == iter.next());
    assert!(Some('o') == iter.next());
    assert!(Some('i') == iter.next());
    assert!(Some('n') == iter.next());
    assert!(Some('g') == iter.next());
    assert!(Some(',') == iter.next());
    assert!(None == iter.next());
}


#[test]
fn slice_8() {
    let rope = Rope::from_str("Hello everyone!  How are you doing, eh?");
    let s = rope.slice(15, 39);
    
    let mut iter = s.char_iter_between_indices(0, 24);
    
    assert!(Some(' ') == iter.next());
    assert!(Some(' ') == iter.next());
    assert!(Some('H') == iter.next());
    assert!(Some('o') == iter.next());
    assert!(Some('w') == iter.next());
    assert!(Some(' ') == iter.next());
    assert!(Some('a') == iter.next());
    assert!(Some('r') == iter.next());
    assert!(Some('e') == iter.next());
    assert!(Some(' ') == iter.next());
    assert!(Some('y') == iter.next());
    assert!(Some('o') == iter.next());
    assert!(Some('u') == iter.next());
    assert!(Some(' ') == iter.next());
    assert!(Some('d') == iter.next());
    assert!(Some('o') == iter.next());
    assert!(Some('i') == iter.next());
    assert!(Some('n') == iter.next());
    assert!(Some('g') == iter.next());
    assert!(Some(',') == iter.next());
    assert!(Some(' ') == iter.next());
    assert!(Some('e') == iter.next());
    assert!(Some('h') == iter.next());
    assert!(Some('?') == iter.next());
    assert!(None == iter.next());
}


#[test]
fn slice_9() {
    let rope = Rope::from_str("Hello everyone!\u{000D}\u{000A}How are you doing, eh?");
    let s = rope.slice(0, 39);
    
    let mut iter = s.grapheme_iter();
    
    assert_eq!(Some("H"), iter.next());
    assert_eq!(Some("e"), iter.next());
    assert_eq!(Some("l"), iter.next());
    assert_eq!(Some("l"), iter.next());
    assert_eq!(Some("o"), iter.next());
    assert_eq!(Some(" "), iter.next());
    assert_eq!(Some("e"), iter.next());
    assert_eq!(Some("v"), iter.next());
    assert_eq!(Some("e"), iter.next());
    assert_eq!(Some("r"), iter.next());
    assert_eq!(Some("y"), iter.next());
    assert_eq!(Some("o"), iter.next());
    assert_eq!(Some("n"), iter.next());
    assert_eq!(Some("e"), iter.next());
    assert_eq!(Some("!"), iter.next());
    assert_eq!(Some("\u{000D}\u{000A}"), iter.next());
    assert_eq!(Some("H"), iter.next());
    assert_eq!(Some("o"), iter.next());
    assert_eq!(Some("w"), iter.next());
    assert_eq!(Some(" "), iter.next());
    assert_eq!(Some("a"), iter.next());
    assert_eq!(Some("r"), iter.next());
    assert_eq!(Some("e"), iter.next());
    assert_eq!(Some(" "), iter.next());
    assert_eq!(Some("y"), iter.next());
    assert_eq!(Some("o"), iter.next());
    assert_eq!(Some("u"), iter.next());
    assert_eq!(Some(" "), iter.next());
    assert_eq!(Some("d"), iter.next());
    assert_eq!(Some("o"), iter.next());
    assert_eq!(Some("i"), iter.next());
    assert_eq!(Some("n"), iter.next());
    assert_eq!(Some("g"), iter.next());
    assert_eq!(Some(","), iter.next());
    assert_eq!(Some(" "), iter.next());
    assert_eq!(Some("e"), iter.next());
    assert_eq!(Some("h"), iter.next());
    assert_eq!(Some("?"), iter.next());
    assert_eq!(None, iter.next());
}


#[test]
fn slice_10() {
    let rope = Rope::from_str("Hello everyone!\u{000D}\u{000A}How are you doing, eh?");
    let s = rope.slice(0, 39);
    
    let mut iter = s.grapheme_iter_at_index(16);
    
    assert_eq!(Some("H"), iter.next());
    assert_eq!(Some("o"), iter.next());
    assert_eq!(Some("w"), iter.next());
    assert_eq!(Some(" "), iter.next());
    assert_eq!(Some("a"), iter.next());
    assert_eq!(Some("r"), iter.next());
    assert_eq!(Some("e"), iter.next());
    assert_eq!(Some(" "), iter.next());
    assert_eq!(Some("y"), iter.next());
    assert_eq!(Some("o"), iter.next());
    assert_eq!(Some("u"), iter.next());
    assert_eq!(Some(" "), iter.next());
    assert_eq!(Some("d"), iter.next());
    assert_eq!(Some("o"), iter.next());
    assert_eq!(Some("i"), iter.next());
    assert_eq!(Some("n"), iter.next());
    assert_eq!(Some("g"), iter.next());
    assert_eq!(Some(","), iter.next());
    assert_eq!(Some(" "), iter.next());
    assert_eq!(Some("e"), iter.next());
    assert_eq!(Some("h"), iter.next());
    assert_eq!(Some("?"), iter.next());
    assert_eq!(None, iter.next());
}


#[test]
fn slice_11() {
    let rope = Rope::from_str("Hello everyone!\u{000D}\u{000A}How are you doing, eh?");
    let s = rope.slice(17, 39);
    
    let mut iter = s.grapheme_iter_at_index(0);
    
    assert_eq!(Some("H"), iter.next());
    assert_eq!(Some("o"), iter.next());
    assert_eq!(Some("w"), iter.next());
    assert_eq!(Some(" "), iter.next());
    assert_eq!(Some("a"), iter.next());
    assert_eq!(Some("r"), iter.next());
    assert_eq!(Some("e"), iter.next());
    assert_eq!(Some(" "), iter.next());
    assert_eq!(Some("y"), iter.next());
    assert_eq!(Some("o"), iter.next());
    assert_eq!(Some("u"), iter.next());
    assert_eq!(Some(" "), iter.next());
    assert_eq!(Some("d"), iter.next());
    assert_eq!(Some("o"), iter.next());
    assert_eq!(Some("i"), iter.next());
    assert_eq!(Some("n"), iter.next());
    assert_eq!(Some("g"), iter.next());
    assert_eq!(Some(","), iter.next());
    assert_eq!(Some(" "), iter.next());
    assert_eq!(Some("e"), iter.next());
    assert_eq!(Some("h"), iter.next());
    assert_eq!(Some("?"), iter.next());
    assert_eq!(None, iter.next());
}


#[test]
fn slice_12() {
    let rope = Rope::from_str("Hello everyone!\u{000D}\u{000A}How are you doing, eh?");
    let s = rope.slice(5, 20);
    
    let mut iter = s.grapheme_iter_between_indices(6, 12);
    
    assert_eq!(Some("o"), iter.next());
    assert_eq!(Some("n"), iter.next());
    assert_eq!(Some("e"), iter.next());
    assert_eq!(Some("!"), iter.next());
    assert_eq!(Some("\u{000D}\u{000A}"), iter.next());
    assert_eq!(Some("H"), iter.next());
    assert_eq!(None, iter.next());
}


#[test]
fn slice_13() {
    let rope = Rope::from_str("Hello everyone!\u{000D}\u{000A}How are you doing, eh?");
    let s = rope.slice(16, 39);
    
    let mut iter = s.grapheme_iter_at_index(0);
    
    assert_eq!(Some("\u{000A}"), iter.next());
    assert_eq!(Some("H"), iter.next());
    assert_eq!(Some("o"), iter.next());
    assert_eq!(Some("w"), iter.next());
    assert_eq!(Some(" "), iter.next());
    assert_eq!(Some("a"), iter.next());
    assert_eq!(Some("r"), iter.next());
    assert_eq!(Some("e"), iter.next());
    assert_eq!(Some(" "), iter.next());
    assert_eq!(Some("y"), iter.next());
    assert_eq!(Some("o"), iter.next());
    assert_eq!(Some("u"), iter.next());
    assert_eq!(Some(" "), iter.next());
    assert_eq!(Some("d"), iter.next());
    assert_eq!(Some("o"), iter.next());
    assert_eq!(Some("i"), iter.next());
    assert_eq!(Some("n"), iter.next());
    assert_eq!(Some("g"), iter.next());
    assert_eq!(Some(","), iter.next());
    assert_eq!(Some(" "), iter.next());
    assert_eq!(Some("e"), iter.next());
    assert_eq!(Some("h"), iter.next());
    assert_eq!(Some("?"), iter.next());
    assert_eq!(None, iter.next());
}


#[test]
fn slice_14() {
    let rope = Rope::from_str("Hello everyone!\u{000D}\u{000A}How are you doing, eh?");
    let s = rope.slice(0, 16);
    
    let mut iter = s.grapheme_iter_at_index(0);
    
    assert_eq!(Some("H"), iter.next());
    assert_eq!(Some("e"), iter.next());
    assert_eq!(Some("l"), iter.next());
    assert_eq!(Some("l"), iter.next());
    assert_eq!(Some("o"), iter.next());
    assert_eq!(Some(" "), iter.next());
    assert_eq!(Some("e"), iter.next());
    assert_eq!(Some("v"), iter.next());
    assert_eq!(Some("e"), iter.next());
    assert_eq!(Some("r"), iter.next());
    assert_eq!(Some("y"), iter.next());
    assert_eq!(Some("o"), iter.next());
    assert_eq!(Some("n"), iter.next());
    assert_eq!(Some("e"), iter.next());
    assert_eq!(Some("!"), iter.next());
    assert_eq!(Some("\u{000D}"), iter.next());
    assert_eq!(None, iter.next());
}


#[test]
fn slice_15() {
    let rope = Rope::from_str("Hello everyone!\u{000D}\u{000A}How are you doing, eh?");
    let s = rope.slice(16, 39);
    
    assert_eq!("\u{000A}", s.grapheme_at_index(0));
}


#[test]
fn slice_16() {
    let rope = Rope::from_str("Hello everyone!\u{000D}\u{000A}How are you doing, eh?");
    let s = rope.slice(0, 16);
    
    assert_eq!("\u{000D}", s.grapheme_at_index(15));
}


#[test]
fn char_index_to_grapheme_index_1() {
    let rope = Rope::from_str("Hello everyone!  How are you doing, eh?"); // 39 chars, 39 graphemes
    
    assert_eq!(rope.char_index_to_grapheme_index(0), 0);
    assert_eq!(rope.char_index_to_grapheme_index(5), 5);
    assert_eq!(rope.char_index_to_grapheme_index(39), 39);
}


#[test]
fn char_index_to_grapheme_index_2() {
    let rope = Rope::from_str("Hello everyone!\u{000D}\u{000A}How are you doing, eh?"); // 39 chars, 38 graphemes
    
    assert_eq!(rope.char_index_to_grapheme_index(0), 0);
    assert_eq!(rope.char_index_to_grapheme_index(15), 15);
    assert_eq!(rope.char_index_to_grapheme_index(16), 15);
    assert_eq!(rope.char_index_to_grapheme_index(17), 16);
    assert_eq!(rope.char_index_to_grapheme_index(39), 38);
}


#[test]
fn grapheme_index_to_char_index_1() {
    let rope = Rope::from_str("Hello everyone!  How are you doing, eh?"); // 39 chars, 39 graphemes
    
    assert_eq!(rope.grapheme_index_to_char_index(0), 0);
    assert_eq!(rope.grapheme_index_to_char_index(5), 5);
    assert_eq!(rope.grapheme_index_to_char_index(39), 39);
}


#[test]
fn grapheme_index_to_char_index_2() {
    let rope = Rope::from_str("Hello everyone!\u{000D}\u{000A}How are you doing, eh?"); // 39 chars, 38 graphemes
    
    assert_eq!(rope.grapheme_index_to_char_index(0), 0);
    assert_eq!(rope.grapheme_index_to_char_index(15), 15);
    assert_eq!(rope.grapheme_index_to_char_index(16), 17);
    assert_eq!(rope.grapheme_index_to_char_index(38), 39);
}


#[test]
fn line_index_to_char_index_1() {
    let rope = Rope::from_str("Hello\nworld!\n");
    
    assert_eq!(rope.line_index_to_char_index(0), 0);
    assert_eq!(rope.line_index_to_char_index(1), 6);
    assert_eq!(rope.line_index_to_char_index(2), 13);
}


#[test]
fn line_index_to_grapheme_index_2() {
    let rope = Rope::from_str("Hi\nthere\npeople\nof\nthe\nworld!");
    
    assert_eq!(rope.line_index_to_char_index(0), 0);
    assert_eq!(rope.line_index_to_char_index(1), 3);
    assert_eq!(rope.line_index_to_char_index(2), 9);
    assert_eq!(rope.line_index_to_char_index(3), 16);
    assert_eq!(rope.line_index_to_char_index(4), 19);
    assert_eq!(rope.line_index_to_char_index(5), 23);
}


#[test]
fn char_index_to_line_index_1() {
    let rope = Rope::from_str("Hello\nworld!\n");
    
    assert_eq!(rope.char_index_to_line_index(0), 0);
    assert_eq!(rope.char_index_to_line_index(1), 0);
    assert_eq!(rope.char_index_to_line_index(5), 0);
    assert_eq!(rope.char_index_to_line_index(6), 1);
    assert_eq!(rope.char_index_to_line_index(12), 1);
    assert_eq!(rope.char_index_to_line_index(13), 2);
}


#[test]
fn char_index_to_line_index_2() {
    let rope = Rope::from_str("Hi\nthere\npeople\nof\nthe\nworld!");
    
    assert_eq!(rope.char_index_to_line_index(0), 0);
    assert_eq!(rope.char_index_to_line_index(2), 0);
    assert_eq!(rope.char_index_to_line_index(3), 1);
    assert_eq!(rope.char_index_to_line_index(8), 1);
    assert_eq!(rope.char_index_to_line_index(9), 2);
    assert_eq!(rope.char_index_to_line_index(15), 2);
    assert_eq!(rope.char_index_to_line_index(16), 3);
    assert_eq!(rope.char_index_to_line_index(18), 3);
    assert_eq!(rope.char_index_to_line_index(19), 4);
    assert_eq!(rope.char_index_to_line_index(22), 4);
    assert_eq!(rope.char_index_to_line_index(23), 5);
    assert_eq!(rope.char_index_to_line_index(29), 5);
}


#[test]
fn to_string() {
    let rope = Rope::from_str("Hello there good people of the world!");
    let s = rope.to_string();
    
    assert_eq!("Hello there good people of the world!", &s[..]);
}


#[test]
fn split_at_char_index_1() {
    let mut rope1 = Rope::from_str("Hello there good people of the world!");
    
    //let mut f1 = BufferedWriter::new(File::create(&Path::new("yar1.gv")).unwrap());
    //f1.write_str(&(rope1.to_graphviz())[..]);
            
    let rope2 = rope1.split_at_char_index(18);

    //let mut f2 = BufferedWriter::new(File::create(&Path::new("yar2.gv")).unwrap());
    //f2.write_str(&(rope1.to_graphviz())[..]);
    //f2.write_str(&(rope2.to_graphviz())[..]);
    
    assert!(rope1.is_balanced());
    assert!(rope2.is_balanced());
    assert_eq!("Hello there good p", &(rope1.to_string())[..]);
    assert_eq!("eople of the world!", &(rope2.to_string())[..]);
}


#[test]
fn split_at_char_index_2() {
    let mut rope1 = Rope::from_str("Hello there good people of the world!");
    
    //let mut f1 = BufferedWriter::new(File::create(&Path::new("yar1.gv")).unwrap());
    //f1.write_str(&(rope1.to_graphviz())[..]);
            
    let rope2 = rope1.split_at_char_index(31);

    //let mut f2 = BufferedWriter::new(File::create(&Path::new("yar2.gv")).unwrap());
    //f2.write_str(&(rope1.to_graphviz())[..]);
    //f2.write_str(&(rope2.to_graphviz())[..]);
    
    assert!(rope1.is_balanced());
    assert!(rope2.is_balanced());
    assert_eq!("Hello there good people of the ", &(rope1.to_string())[..]);
    assert_eq!("world!", &(rope2.to_string())[..]);
}


#[test]
fn split_at_char_index_3() {
    let mut rope1 = Rope::from_str("Hello there good people of the world!");
    
    //let mut f1 = BufferedWriter::new(File::create(&Path::new("yar1.gv")).unwrap());
    //f1.write_str(&(rope1.to_graphviz())[..]);
            
    let rope2 = rope1.split_at_char_index(5);

    //let mut f2 = BufferedWriter::new(File::create(&Path::new("yar2.gv")).unwrap());
    //f2.write_str(&(rope1.to_graphviz())[..]);
    //f2.write_str(&(rope2.to_graphviz())[..]);
    
    assert!(rope1.is_balanced());
    assert!(rope2.is_balanced());
    assert_eq!("Hello", &(rope1.to_string())[..]);
    assert_eq!(" there good people of the world!", &(rope2.to_string())[..]);
}


#[test]
fn split_at_char_index_4() {
    let mut rope1 = Rope::from_str("Hello there good people of the world!");
    let rope2 = rope1.split_at_char_index(37);
    
    assert!(rope1.is_balanced());
    assert!(rope2.is_balanced());
    assert_eq!("Hello there good people of the world!", &(rope1.to_string())[..]);
    assert_eq!("", &(rope2.to_string())[..]);
}


#[test]
fn split_at_char_index_5() {
    let mut rope1 = Rope::from_str("Hello there good people of the world!");
    let rope2 = rope1.split_at_char_index(0);
    
    assert!(rope1.is_balanced());
    assert!(rope2.is_balanced());
    assert_eq!("", &(rope1.to_string())[..]);
    assert_eq!("Hello there good people of the world!", &(rope2.to_string())[..]);
}


#[test]
fn split_at_char_index_6() {
    let mut rope1 = Rope::from_str("Hello there good\u{000D}\u{000A}people of the world!");
    let rope2 = rope1.split_at_char_index(17);
    
    assert!(rope1.is_balanced());
    assert!(rope2.is_balanced());
    assert_eq!("Hello there good\u{000D}", &(rope1.to_string())[..]);
    assert_eq!("\u{000A}people of the world!", &(rope2.to_string())[..]);
}


#[test]
fn append_1() {
    let mut rope1 = Rope::from_str("Hello there good p");
    let rope2 = Rope::from_str("eople of the world!");
    
    rope1.append(rope2);
    
    assert!(rope1.is_balanced());
    assert_eq!("Hello there good people of the world!", &(rope1.to_string())[..]);
}


#[test]
fn append_2() {
    let mut rope1 = Rope::from_str("Hello there good people of the world!");
    let rope2 = Rope::from_str("");
    
    rope1.append(rope2);
    
    assert!(rope1.is_balanced());
    assert_eq!("Hello there good people of the world!", &(rope1.to_string())[..]);
}


#[test]
fn append_3() {
    let mut rope1 = Rope::from_str("");
    let rope2 = Rope::from_str("Hello there good people of the world!");
    
    rope1.append(rope2);
    
    assert!(rope1.is_balanced());
    assert_eq!("Hello there good people of the world!", &(rope1.to_string())[..]);
}


#[test]
fn append_4() {
    let mut rope1 = Rope::from_str("1234567890-=qwertyuiop{}asdfghjkl;'zxcvbnm,.Hello World!  Let's make this a long string for kicks and giggles.  Who knows when it will end?  No one!  Well, except for the person writing it.  And... eh... later, the person reading it.  Because they'll get to the end.  And then they'll know.");
    let rope2 = Rope::from_str("Z");
    
    rope1.append(rope2);
    
    assert!(rope1.is_balanced());
    assert_eq!(rope1.to_string(), "1234567890-=qwertyuiop{}asdfghjkl;'zxcvbnm,.Hello World!  Let's make this a long string for kicks and giggles.  Who knows when it will end?  No one!  Well, except for the person writing it.  And... eh... later, the person reading it.  Because they'll get to the end.  And then they'll know.Z");
}


#[test]
fn append_5() {
    let mut rope1 = Rope::from_str("Z");
    let rope2 = Rope::from_str("1234567890-=qwertyuiop{}asdfghjkl;'zxcvbnm,.Hello World!  Let's make this a long string for kicks and giggles.  Who knows when it will end?  No one!  Well, except for the person writing it.  And... eh... later, the person reading it.  Because they'll get to the end.  And then they'll know.");
    
    rope1.append(rope2);
    
    assert!(rope1.is_balanced());
    assert_eq!(rope1.to_string(), "Z1234567890-=qwertyuiop{}asdfghjkl;'zxcvbnm,.Hello World!  Let's make this a long string for kicks and giggles.  Who knows when it will end?  No one!  Well, except for the person writing it.  And... eh... later, the person reading it.  Because they'll get to the end.  And then they'll know.");
}


#[test]
fn append_6() {
    let mut rope1 = Rope::from_str("Hello there everyone!\u{000D}");
    let rope2 = Rope::from_str("\u{000A}How is everyone doing?");
    
    assert_eq!(rope1.grapheme_count(), 22);
    assert_eq!(rope2.grapheme_count(), 23);
    
    rope1.append(rope2);
    
    assert_eq!(rope1.to_string(), "Hello there everyone!\u{000D}\u{000A}How is everyone doing?");
    assert_eq!(rope1.grapheme_count(), 44);
    assert_eq!(rope1.grapheme_at_index(21), "\u{000D}\u{000A}");
}


#[test]
fn insert_text_at_char_index_1() {
    let mut rope = Rope::from_str("Hello there!\u{000D}\u{000A}How are you?");
    
    rope.insert_text_at_char_index("Z", 0);
    
    assert_eq!(rope.to_string(), "ZHello there!\u{000D}\u{000A}How are you?".to_string());
}


#[test]
fn insert_text_at_char_index_2() {
    let mut rope = Rope::from_str("Hello there!\u{000D}\u{000A}How are you?");
    
    rope.insert_text_at_char_index("Z", 12);
    
    assert_eq!(rope.to_string(), "Hello there!Z\u{000D}\u{000A}How are you?".to_string());
}


#[test]
fn insert_text_at_char_index_3() {
    let mut rope = Rope::from_str("Hello there!\u{000D}\u{000A}How are you?");
    
    rope.insert_text_at_char_index("Z", 13);
    
    assert_eq!(rope.to_string(), "Hello there!\u{000D}Z\u{000A}How are you?".to_string());
}


#[test]
fn insert_text_at_char_index_4() {
    let mut rope = Rope::from_str("Hello there!\u{000D}\u{000A}How are you?");
    
    rope.insert_text_at_char_index("Z", 14);
    
    assert_eq!(rope.to_string(), "Hello there!\u{000D}\u{000A}ZHow are you?".to_string());
}


#[test]
fn insert_text_at_char_index_5() {
    let mut rope = Rope::from_str("Hello there!\u{000D}\u{000A}How are you?");
    
    rope.insert_text_at_char_index("Z", 26);
    
    assert_eq!(rope.to_string(), "Hello there!\u{000D}\u{000A}How are you?Z".to_string());
}


#[test]
fn insert_text_at_char_index_6() {
    let mut s = String::from_utf8(vec!['c' as u8; (MAX_NODE_SIZE*47) - 1]).unwrap();
    s.push_str("\u{000D}");
    
    let mut rope = Rope::from_str(&s[..]);
    
    s.push_str("\u{000A}");
    rope.insert_text_at_char_index("\u{000A}", MAX_NODE_SIZE*47);
    
    assert_eq!(rope.to_string(), s);
    assert_eq!(rope.grapheme_count(), MAX_NODE_SIZE * 47);
    assert_eq!(rope.grapheme_at_index((MAX_NODE_SIZE * 47) - 1), "\u{000D}\u{000A}");
}


#[test]
fn remove_text_between_char_indices_1() {
    let mut rope = Rope::from_str("Hello there!\u{000D}\u{000A}How are you?");
    
    rope.remove_text_between_char_indices(0, 1);
    
    assert_eq!(rope.to_string(), "ello there!\u{000D}\u{000A}How are you?".to_string());
}


#[test]
fn remove_text_between_char_indices_2() {
    let mut rope = Rope::from_str("Hello there!\u{000D}\u{000A}How are you?");
    
    rope.remove_text_between_char_indices(12, 13);
    
    assert_eq!(rope.to_string(), "Hello there!\u{000A}How are you?".to_string());
}


#[test]
fn remove_text_between_char_indices_3() {
    let mut rope = Rope::from_str("Hello there!\u{000D}\u{000A}How are you?");
    
    rope.remove_text_between_char_indices(13, 14);
    
    assert_eq!(rope.to_string(), "Hello there!\u{000D}How are you?".to_string());
}


#[test]
fn remove_text_between_char_indices_4() {
    let mut s = String::from_utf8(vec!['c' as u8; (MAX_NODE_SIZE*27) - 1]).unwrap();
    s.push_str("\u{000D}");
    s.push_str("Hello there!\u{000A}How are you doing?");
    
    let mut rope = Rope::from_str(&s[..]);
    
    rope.remove_text_between_char_indices((MAX_NODE_SIZE*27), (MAX_NODE_SIZE*27)+12);
    
    remove_text_between_char_indices(&mut s, (MAX_NODE_SIZE*27), (MAX_NODE_SIZE*27)+12);
    
    assert_eq!(rope.to_string(), s);
    assert_eq!(rope.grapheme_count(), (MAX_NODE_SIZE * 27) + 18);
    assert_eq!(rope.grapheme_at_index((MAX_NODE_SIZE * 27) - 1), "\u{000D}\u{000A}");
}


#[test]
fn insert_text() {
    let mut rope = Rope::new();
    
    rope.insert_text_at_char_index("Hello 世界!", 0);
    
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
    let mut rope = Rope::from_str("Hello\n 世界\r\n!");
    
    rope.insert_text_at_char_index("Again ", 0);
    
    let mut iter = rope.grapheme_iter();
    
    assert!(rope.is_balanced());
    assert_eq!(rope.grapheme_count(), 17);
    assert_eq!(rope.line_ending_count(), 2);
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
    let mut rope = Rope::from_str("Hello\n 世界\r\n!");
    
    rope.insert_text_at_char_index(" again", 5);
    
    let mut iter = rope.grapheme_iter();
    
    assert!(rope.is_balanced());
    assert_eq!(rope.grapheme_count(), 17);
    assert_eq!(rope.line_ending_count(), 2);
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
    let mut rope = Rope::from_str("Hello\n 世界\r\n!");
    
    rope.insert_text_at_char_index("again", 6);
    
    let mut iter = rope.grapheme_iter();
    
    assert!(rope.is_balanced());
    assert_eq!(rope.grapheme_count(), 16);
    assert_eq!(rope.line_ending_count(), 2);
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
    let mut rope = Rope::from_str("Hello\n 世界\r\n!");        

    rope.insert_text_at_char_index("again", 12);
    
    let mut iter = rope.grapheme_iter();
    
    assert!(rope.is_balanced());
    assert_eq!(rope.grapheme_count(), 16);
    assert_eq!(rope.line_ending_count(), 2);
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
    let mut rope = Rope::from_str("Hello\n 世界\r\n!");
    
    rope.insert_text_at_char_index("again", 2);
    
    let mut iter = rope.grapheme_iter();
    
    assert!(rope.is_balanced());
    assert_eq!(rope.grapheme_count(), 16);
    assert_eq!(rope.line_ending_count(), 2);
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
    let mut rope = Rope::from_str("Hello\n 世界\r\n!");
    
    rope.insert_text_at_char_index("again", 8);
    
    let mut iter = rope.grapheme_iter();
    
    assert!(rope.is_balanced());
    assert_eq!(rope.grapheme_count(), 16);
    assert_eq!(rope.line_ending_count(), 2);
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
    let mut rope = Rope::from_str("Hello\n 世界\r\n!");
    
    rope.insert_text_at_char_index("\nag\n\nain\n", 2);
    
    let mut iter = rope.grapheme_iter();
    
    assert!(rope.is_balanced());
    assert_eq!(rope.grapheme_count(), 20);
    assert_eq!(rope.line_ending_count(), 6);
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
    let mut rope = Rope::from_str("Hi\nthere\npeople\nof\nthe\nworld!");
    
    rope.remove_text_between_char_indices(0, 3);
    
    let mut iter = rope.grapheme_iter();
    
    assert!(rope.is_balanced());
    assert_eq!(rope.grapheme_count(), 26);
    assert_eq!(rope.line_ending_count(), 4);
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
    let mut rope = Rope::from_str("Hi\nthere\npeople\nof\nthe\nworld!");
    
    rope.remove_text_between_char_indices(0, 12);
    
    let mut iter = rope.grapheme_iter();
    
    assert!(rope.is_balanced());
    assert_eq!(rope.grapheme_count(), 17);
    assert_eq!(rope.line_ending_count(), 3);
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
    let mut rope = Rope::from_str("Hi\nthere\npeople\nof\nthe\nworld!");
    
    rope.remove_text_between_char_indices(5, 17);
    
    let mut iter = rope.grapheme_iter();
    
    assert!(rope.is_balanced());
    assert_eq!(rope.grapheme_count(), 17);
    assert_eq!(rope.line_ending_count(), 3);
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
    let mut rope = Rope::from_str("Hi\nthere\npeople\nof\nthe\nworld!");
    
    rope.remove_text_between_char_indices(23, 29);
    
    let mut iter = rope.grapheme_iter();
    
    assert!(rope.is_balanced());
    assert_eq!(rope.grapheme_count(), 23);
    assert_eq!(rope.line_ending_count(), 5);
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
    let mut rope = Rope::from_str("Hi\nthere\npeople\nof\nthe\nworld!");
    
    rope.remove_text_between_char_indices(17, 29);
    
    let mut iter = rope.grapheme_iter();
    
    assert!(rope.is_balanced());
    assert_eq!(rope.grapheme_count(), 17);
    assert_eq!(rope.line_ending_count(), 3);
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
    let mut rope = Rope::from_str("Hello\nworld!");
    
    rope.remove_text_between_char_indices(3, 12);
    
    let mut iter = rope.grapheme_iter();
    
    assert!(rope.is_balanced());
    assert_eq!(rope.grapheme_count(), 3);
    assert_eq!(rope.line_ending_count(), 0);
    assert!(Some("H") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some("l") == iter.next());
    assert!(None == iter.next());
}


#[test]
fn remove_text_7() {
    let mut rope = Rope::from_str("Hi\nthere\nworld!");
    
    rope.remove_text_between_char_indices(5, 15);
    
    let mut iter = rope.grapheme_iter();
    
    assert!(rope.is_balanced());
    assert_eq!(rope.grapheme_count(), 5);
    assert_eq!(rope.line_ending_count(), 1);
    assert!(Some("H") == iter.next());
    assert!(Some("i") == iter.next());
    assert!(Some("\n") == iter.next());
    assert!(Some("t") == iter.next());
    assert!(Some("h") == iter.next());
    assert!(None == iter.next());
}


#[test]
fn remove_text_8() {
    let mut rope = Rope::from_str("Hello\nworld!");
    
    rope.remove_text_between_char_indices(3, 11);
    
    let mut iter = rope.grapheme_iter();
    
    assert!(rope.is_balanced());
    assert_eq!(rope.grapheme_count(), 4);
    assert_eq!(rope.line_ending_count(), 0);
    assert!(Some("H") == iter.next());
    assert!(Some("e") == iter.next());
    assert!(Some("l") == iter.next());
    assert!(Some("!") == iter.next());
    assert!(None == iter.next());
}


#[test]
fn remove_text_9() {
    let mut rope = Rope::from_str("Hello\nworld!");
    
    rope.remove_text_between_char_indices(8, 12);
    
    let mut iter = rope.grapheme_iter();
    
    assert!(rope.is_balanced());
    assert_eq!(rope.grapheme_count(), 8);
    assert_eq!(rope.line_ending_count(), 1);
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
    let mut rope = Rope::from_str("12\n34\n56\n78");
    
    rope.remove_text_between_char_indices(4, 11);
    
    let mut iter = rope.grapheme_iter();
    
    assert!(rope.is_balanced());
    assert_eq!(rope.grapheme_count(), 4);
    assert_eq!(rope.line_ending_count(), 1);
    assert!(Some("1") == iter.next());
    assert!(Some("2") == iter.next());
    assert!(Some("\n") == iter.next());
    assert!(Some("3") == iter.next());
    assert!(None == iter.next());
}


#[test]
fn remove_text_11() {
    let mut rope = Rope::from_str("1234567890");
    
    rope.remove_text_between_char_indices(9, 10);
    
    let mut iter = rope.grapheme_iter();
    
    assert!(rope.is_balanced());
    assert_eq!(rope.grapheme_count(), 9);
    assert_eq!(rope.line_ending_count(), 0);
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
    let left = Rope::from_str(&(String::from_utf8(vec!['c' as u8; MAX_NODE_SIZE * 64]).unwrap())[..]);
    let right = Rope::from_str(&(String::from_utf8(vec!['c' as u8; MAX_NODE_SIZE * 1]).unwrap())[..]);
    
    let mut rope = Rope {
        data: RopeData::Branch(Box::new(left), Box::new(right)),
        char_count_: 0,
        grapheme_count_: 0,
        line_ending_count_: 0,
        tree_height: 1,
    };
    rope.update_stats();
    
    //let mut f1 = BufferedWriter::new(File::create(&Path::new("yar1.gv")).unwrap());
    //f1.write_str(&(rope.to_graphviz())[..]);
    
    rope.rebalance();
    
    //let mut f2 = BufferedWriter::new(File::create(&Path::new("yar2.gv")).unwrap());
    //f2.write_str(&(rope.to_graphviz())[..]);
    
    assert!(rope.is_balanced());
}


#[test]
fn rebalance_2() {
    let left = Rope::from_str(&(String::from_utf8(vec!['c' as u8; MAX_NODE_SIZE * 1]).unwrap())[..]);
    let right = Rope::from_str(&(String::from_utf8(vec!['c' as u8; MAX_NODE_SIZE * 64]).unwrap())[..]);
    
    let mut rope = Rope {
        data: RopeData::Branch(Box::new(left), Box::new(right)),
        char_count_: 0,
        grapheme_count_: 0,
        line_ending_count_: 0,
        tree_height: 1,
    };
    rope.update_stats();
    
    //let mut f1 = BufferedWriter::new(File::create(&Path::new("yar1.gv")).unwrap());
    //f1.write_str(&(rope.to_graphviz())[..]);
    
    rope.rebalance();
    
    //let mut f2 = BufferedWriter::new(File::create(&Path::new("yar2.gv")).unwrap());
    //f2.write_str(&(rope.to_graphviz())[..]);
    
    assert!(rope.is_balanced());
}


#[test]
fn rebalance_3() {
    let left = Rope::from_str(&(String::from_utf8(vec!['c' as u8; MAX_NODE_SIZE * 53]).unwrap())[..]);
    let right = Rope::from_str(&(String::from_utf8(vec!['c' as u8; MAX_NODE_SIZE * 1]).unwrap())[..]);
    
    let mut rope = Rope {
        data: RopeData::Branch(Box::new(left), Box::new(right)),
        char_count_: 0,
        grapheme_count_: 0,
        line_ending_count_: 0,
        tree_height: 1,
    };
    rope.update_stats();
    
    //let mut f1 = BufferedWriter::new(File::create(&Path::new("yar1.gv")).unwrap());
    //f1.write_str(&(rope.to_graphviz())[..]);
    
    rope.rebalance();
    
    //let mut f2 = BufferedWriter::new(File::create(&Path::new("yar2.gv")).unwrap());
    //f2.write_str(&(rope.to_graphviz())[..]);
    
    assert!(rope.is_balanced());
}


#[test]
fn rebalance_4() {
    let left = Rope::from_str(&(String::from_utf8(vec!['c' as u8; MAX_NODE_SIZE * 1]).unwrap())[..]);
    let right = Rope::from_str(&(String::from_utf8(vec!['c' as u8; MAX_NODE_SIZE * 53]).unwrap())[..]);
    
    let mut rope = Rope {
        data: RopeData::Branch(Box::new(left), Box::new(right)),
        char_count_: 0,
        grapheme_count_: 0,
        line_ending_count_: 0,
        tree_height: 1,
    };
    rope.update_stats();
    
    //let mut f1 = BufferedWriter::new(File::create(&Path::new("yar1.gv")).unwrap());
    //f1.write_str(&(rope.to_graphviz())[..]);
    
    rope.rebalance();
    
    //let mut f2 = BufferedWriter::new(File::create(&Path::new("yar2.gv")).unwrap());
    //f2.write_str(&(rope.to_graphviz())[..]);

    assert!(rope.is_balanced());
}
