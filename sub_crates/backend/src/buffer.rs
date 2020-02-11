use ropey::Rope;

use crate::marks::MarkSet;

/// An open text buffer, currently being edited.
#[derive(Debug, Clone)]
pub struct Buffer {
    pub is_dirty: bool,          // Is this buffer currently out of sync with disk.
    pub text: Rope,              // The actual text content.
    pub mark_sets: Vec<MarkSet>, // MarkSets for cursors, view positions, etc.
}

impl Buffer {
    pub fn new(text: Rope) -> Buffer {
        Buffer {
            is_dirty: false,
            text: text,
            mark_sets: Vec::new(),
        }
    }

    // Replaces the given range of chars with the given text.
    pub fn edit(&mut self, char_idx_range: (usize, usize), text: &str) {
        self.is_dirty = true;

        // Get the range, properly ordered.
        let (start, end) = if char_idx_range.0 < char_idx_range.1 {
            (char_idx_range.0, char_idx_range.1)
        } else {
            (char_idx_range.1, char_idx_range.0)
        };

        // Do removal if needed.
        if start != end {
            self.text.remove(start..end);
        }

        // Do insertion if needed.
        if !text.is_empty() {
            self.text.insert(start, text);
        }
    }

    /// Creates a new empty mark set, and returns the set index.
    pub fn add_mark_set(&mut self) -> usize {
        self.mark_sets.push(MarkSet::new());
        return self.mark_sets.len() - 1;
    }
}
