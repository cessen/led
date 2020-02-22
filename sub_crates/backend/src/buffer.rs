use ropey::Rope;

use crate::{
    history::{Edit, History},
    marks::MarkSet,
};

/// An open text buffer, currently being edited.
#[derive(Debug, Clone)]
pub struct Buffer {
    pub is_dirty: bool,          // Is this buffer currently out of sync with disk.
    pub text: Rope,              // The actual text content.
    pub mark_sets: Vec<MarkSet>, // MarkSets for cursors, view positions, etc.
    history: History,
}

impl Buffer {
    pub fn new(text: Rope) -> Buffer {
        Buffer {
            is_dirty: false,
            text: text,
            mark_sets: Vec::new(),
            history: History::new(),
        }
    }

    /// Replaces the given range of chars with the given text.
    ///
    /// The range does not have to be ordered (i.e. the first component can be
    /// greater than the second).
    pub fn edit(&mut self, char_idx_range: (usize, usize), text: &str) {
        self.is_dirty = true;

        // Get the range, properly ordered.
        let (start, end) = if char_idx_range.0 < char_idx_range.1 {
            (char_idx_range.0, char_idx_range.1)
        } else {
            (char_idx_range.1, char_idx_range.0)
        };

        // Update undo stack.
        if char_idx_range.0 == char_idx_range.1 {
            // Fast-path for insertion-only edits.
            self.history.push_edit(Edit {
                char_idx: start,
                from: String::new(),
                to: text.into(),
            });
        } else {
            self.history.push_edit(Edit {
                char_idx: start,
                from: self.text.slice(start..end).into(),
                to: text.into(),
            });
        }

        // Update mark sets.
        for mark_set in self.mark_sets.iter_mut() {
            for mark in mark_set.marks.iter_mut() {
                *mark = mark.edit((start, end), text.chars().count());
            }

            mark_set.merge_touching();
        }

        // Do removal if needed.
        if start != end {
            self.text.remove(start..end);
        }

        // Do insertion if needed.
        if !text.is_empty() {
            self.text.insert(start, text);
        }
    }

    pub fn undo(&mut self) {
        todo!()
    }

    pub fn redo(&mut self) {
        todo!()
    }

    /// Creates a new empty mark set, and returns the set index.
    pub fn add_mark_set(&mut self) -> usize {
        self.mark_sets.push(MarkSet::new());
        return self.mark_sets.len() - 1;
    }
}
