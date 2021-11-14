use std::ops::Range;

use ropey::Rope;

use crate::marks::MarkSet;

#[derive(Debug, Clone)]
enum Op {
    Retain {
        byte_count: usize,
    },
    Replace {
        // These both represent strings, and are byte-index ranges into
        // the Transaction's `buffer` where their actual string data is
        // stored.  `0..0` can be used to indicate no string data.
        old: Range<usize>,
        new: Range<usize>,
    },
}

/// A reversable set of edits treated as an atomic unit.
#[derive(Debug, Clone)]
pub struct Transaction {
    ops: Vec<Op>,
    buffer: String,
}

impl Transaction {
    pub fn new() -> Transaction {
        Transaction {
            ops: Vec::new(),
            buffer: String::new(),
        }
    }

    /// Adds another edit to the Transaction.
    ///
    /// This composes as if the edits already in the Transaction were
    /// performed first, and then the new edit was performed after.
    pub fn push_edit(&mut self, byte_idx: usize, old: &str, new: &str) {
        if self.ops.is_empty() {
            // The easy case.
            self.buffer.push_str(old);
            self.buffer.push_str(new);
            self.ops.push(Op::Retain {
                byte_count: byte_idx,
            });
            self.ops.push(Op::Replace {
                old: 0..old.len(),
                new: old.len()..(old.len() + new.len()),
            });
        } else {
            // The complex case.
            todo!()
        }
    }

    /// Build a Transaction that is functionally identical to applying
    /// first `self` and then `other` sequentially.
    #[must_use]
    pub fn compose(&self, _other: &Transaction) -> Transaction {
        todo!()
    }

    /// Build a Transaction that is functionally identical to undoing
    /// this Transaction.
    ///
    /// Note: the resulting Transaction will losslessly reverse the
    /// original Transaction on text content, but will be lossy when
    /// applied to Marks.
    #[must_use]
    pub fn invert(&self) -> Transaction {
        let mut inverted = self.clone();
        for op in inverted.ops.iter_mut() {
            match *op {
                Op::Retain { .. } => {} // Do nothing.
                Op::Replace {
                    ref mut old,
                    ref mut new,
                } => {
                    std::mem::swap(old, new);
                }
            }
        }
        inverted
    }

    /// Applies the Transaction to a Rope.
    pub fn apply_to_text(&self, text: &mut Rope) {
        let mut i = 0;
        for op in self.ops.iter() {
            match op {
                Op::Retain { byte_count } => {
                    i += byte_count;
                }
                Op::Replace { old, new } => {
                    let old = &self.buffer[old.clone()];
                    let new = &self.buffer[new.clone()];
                    let char_i = text.byte_to_char(i);
                    let old_char_len = old.chars().count();

                    debug_assert_eq!(text.slice(char_i..(char_i + old_char_len)), old);
                    text.remove(char_i..(char_i + old_char_len));
                    text.insert(char_i, new);

                    i = i + new.len();
                }
            }
        }
        debug_assert!(i <= text.len_bytes());
    }

    /// Applies the Transaction to a set of Marks.
    pub fn apply_to_marks(&self, _marks: &mut MarkSet) {
        todo!()
    }
}
