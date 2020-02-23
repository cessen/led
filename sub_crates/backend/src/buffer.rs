use std::path::PathBuf;

use ropey::Rope;

use crate::{
    history::{Edit, History},
    marks::MarkSet,
};

/// A path for an open text buffer.
///
/// This indicates where the text data of the buffer came from, and
/// where it should be saved to.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum BufferPath {
    File(PathBuf), // A buffer for a normal file on disk.
    Temp(usize),   // A temporary buffer, with a number ID.
}

/// An open text buffer, currently being edited.
#[derive(Debug, Clone)]
pub struct Buffer {
    pub path: BufferPath,
    pub is_dirty: bool,          // Is this buffer currently out of sync with disk.
    pub text: Rope,              // The actual text content.
    pub mark_sets: Vec<MarkSet>, // MarkSets for cursors, view positions, etc.
    history: History,
}

impl Buffer {
    pub fn new(text: Rope, path: BufferPath) -> Buffer {
        Buffer {
            path: path,
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
        let post_len = text.chars().count();
        for mark_set in self.mark_sets.iter_mut() {
            for mark in mark_set.iter_mut() {
                *mark = mark.edit((start, end), post_len);
            }

            mark_set.make_consistent();
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

    /// Un-does the last edit if there is one, and returns the range of the
    /// edited characters which can be used for e.g. placing a cursor or moving
    /// the view.
    ///
    /// Returns None if there is no edit to undo.
    pub fn undo(&mut self) -> Option<(usize, usize)> {
        if let Some(ed) = self.history.undo() {
            self.is_dirty = true;

            let pre_len = ed.to.chars().count();
            let post_len = ed.from.chars().count();
            let (start, end) = (ed.char_idx, ed.char_idx + pre_len);

            // Update mark sets.
            for mark_set in self.mark_sets.iter_mut() {
                for mark in mark_set.iter_mut() {
                    *mark = mark.edit((start, end), post_len);
                }

                mark_set.make_consistent();
            }

            // Do removal if needed.
            if start != end {
                self.text.remove(start..end);
            }

            // Do insertion if needed.
            if !ed.from.is_empty() {
                self.text.insert(start, &ed.from);
            }

            return Some((start, start + post_len));
        } else {
            return None;
        }
    }

    /// Re-does the last edit if there is one, and returns the range of the
    /// edited characters which can be used for e.g. placing a cursor or moving
    /// the view.
    ///
    /// Returns None if there is no edit to redo.
    pub fn redo(&mut self) -> Option<(usize, usize)> {
        if let Some(ed) = self.history.redo() {
            self.is_dirty = true;

            let pre_len = ed.from.chars().count();
            let post_len = ed.to.chars().count();
            let (start, end) = (ed.char_idx, ed.char_idx + pre_len);

            // Update mark sets.
            for mark_set in self.mark_sets.iter_mut() {
                for mark in mark_set.iter_mut() {
                    *mark = mark.edit((start, end), post_len);
                }

                mark_set.make_consistent();
            }

            // Do removal if needed.
            if start != end {
                self.text.remove(start..end);
            }

            // Do insertion if needed.
            if !ed.to.is_empty() {
                self.text.insert(start, &ed.to);
            }

            return Some((start, start + post_len));
        } else {
            return None;
        }
    }

    /// Creates a new empty mark set, and returns the set index.
    pub fn add_mark_set(&mut self) -> usize {
        self.mark_sets.push(MarkSet::new());
        return self.mark_sets.len() - 1;
    }
}
