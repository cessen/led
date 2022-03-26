use ropey::Rope;

use crate::marks::MarkSet;

#[derive(Debug, Clone, Copy)]
enum Op {
    Retain {
        byte_count: usize,
    },
    Replace {
        // These both represent strings, and are byte-index ranges into
        // the Transaction's `buffer` where their actual string data is
        // stored.  `(0, 0)` can be used to indicate no string data.
        old: (usize, usize),
        new: (usize, usize),
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

    /// Creates a Transaction from a single edit.
    pub fn from_edit(byte_idx: usize, old: &str, new: &str) -> Transaction {
        let mut buffer = String::new();
        buffer.push_str(old);
        buffer.push_str(new);

        let ops = vec![
            Op::Retain {
                byte_count: byte_idx,
            },
            Op::Replace {
                old: (0, old.len()),
                new: (old.len(), old.len() + new.len()),
            },
        ];

        Transaction {
            ops: ops,
            buffer: buffer,
        }
    }

    /// Creates a Transaction from a sorted, non-overlapping set of
    /// simultaneous edits.
    ///
    /// Takes an iterator that yields `(byte_index, old_text, replacement_text)`
    /// tuples, representing the edits.  The items are expected to be
    /// yielded in byte-index order, and the byte indices are relative to
    /// the original text--this function does _not_ treat them as a
    /// sequence of progressively applied edits, but rather as a set of
    /// edits applied simultaneously.
    pub fn from_ordered_edit_set<'a, I>(edit_iter: I) -> Transaction
    where
        I: Iterator<Item = (usize, &'a str, &'a str)> + 'a,
    {
        let mut trans = Transaction::new();
        let mut i = 0;
        let mut len_delta = 0isize;
        for (byte_idx, old, new) in edit_iter {
            let adjusted_byte_idx = (byte_idx as isize + len_delta) as usize;
            let retained = adjusted_byte_idx - i;

            let old_range = (trans.buffer.len(), trans.buffer.len() + old.len());
            trans.buffer.push_str(old);
            let new_range = (trans.buffer.len(), trans.buffer.len() + new.len());
            trans.buffer.push_str(new);

            if retained > 0 {
                trans.ops.push(Op::Retain {
                    byte_count: retained,
                });
            }
            trans.ops.push(Op::Replace {
                old: old_range,
                new: new_range,
            });

            i += retained + new.len();
            len_delta += new.len() as isize - old.len() as isize;
        }
        trans
    }

    /// Build a Transaction that is functionally identical to applying
    /// first `self` and then `other` sequentially.
    #[must_use]
    pub fn compose(&self, other: &Transaction) -> Transaction {
        use Op::*;

        let mut trans = Transaction::new();
        let mut ops1 = self.ops.iter();
        let mut ops2 = other.ops.iter();
        let mut op1 = ops1.next();
        let mut op2 = ops2.next();
        let mut range1 = (0, 0);
        let mut range2 = (0, 0);

        loop {
            match (op1, op2) {
                //-------------------------------
                // Move past the unchanged bits.
                (Some(Retain { byte_count: bc }), _) => {
                    range1.1 += bc;
                    op1 = ops1.next();
                }

                (_, Some(Retain { byte_count: bc })) => {
                    range2.1 += bc;
                    op2 = ops2.next();
                }

                //-----------------------------------------------------
                // Handle changes when there are still changes in both
                // source transactions.
                (
                    Some(Replace {
                        old: old1,
                        new: new1,
                    }),
                    Some(Replace {
                        old: old2,
                        new: new2,
                    }),
                ) => {
                    // This is the complex case.
                    todo!()
                }

                //------------------------------------------------------
                // Handle changes when there are only changes remaining
                // in one of the source transactions.
                (Some(Replace { old, new }), None) => {
                    if range1.0 < range1.1 {
                        trans.ops.push(Retain {
                            byte_count: range1.1 - range1.0,
                        });
                        range1.0 = range1.1;
                    }

                    let ti = trans.buffer.len();
                    trans.buffer.push_str(&self.buffer[old.0..old.1]);
                    let old_range = (ti, ti + (old.1 - old.0));

                    let ti = trans.buffer.len();
                    trans.buffer.push_str(&self.buffer[new.0..new.1]);
                    let new_range = (ti, ti + (new.1 - new.0));

                    trans.ops.push(Replace {
                        old: old_range,
                        new: new_range,
                    });

                    op1 = ops1.next();
                }

                (None, Some(Replace { old, new })) => {
                    if range2.0 < range2.1 {
                        trans.ops.push(Retain {
                            byte_count: range2.1 - range2.0,
                        });
                        range2.0 = range2.1;
                    }

                    let ti = trans.buffer.len();
                    trans.buffer.push_str(&other.buffer[old.0..old.1]);
                    let old_range = (ti, ti + (old.1 - old.0));

                    let ti = trans.buffer.len();
                    trans.buffer.push_str(&other.buffer[new.0..new.1]);
                    let new_range = (ti, ti + (new.1 - new.0));

                    trans.ops.push(Replace {
                        old: old_range,
                        new: new_range,
                    });

                    op2 = ops2.next();
                }

                //-------
                // Done.
                (None, None) => {
                    break;
                }
            }
        }

        trans
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
                    let old = &self.buffer[old.0..old.1];
                    let new = &self.buffer[new.0..new.1];
                    let char_i = text.byte_to_char(i);

                    if !old.is_empty() {
                        let old_char_len = old.chars().count();
                        debug_assert_eq!(text.slice(char_i..(char_i + old_char_len)), old);
                        text.remove(char_i..(char_i + old_char_len));
                    }
                    if !new.is_empty() {
                        text.insert(char_i, new);
                    }

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
