use ropey::Rope;

/// An open text buffer, currently being edited.
#[derive(Debug, Clone)]
pub struct Buffer {
    pub is_dirty: bool, // Is this buffer currently out of sync with disk.
    pub text: Rope,     // The actual text content.

    // Sets of marked ranges to be used for various purposes.
    //
    // Each individual mark consists of a head and a tail.  The ordering of the
    // head and tail is unspecified: either the head or the tail may come first
    // in the text.  Both head and tail are specified in absolute indices.
    //
    // Within sets, marks cannot overlap or abutt, and they must be ordered.
    // Overlapping or abutting marks will be merged, and out-of-order marks
    // will be removed.
    pub mark_sets: Vec<Vec<(usize, usize)>>,
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
        self.mark_sets.push(vec![(0, 0)]);
        return self.mark_sets.len() - 1;
    }

    pub fn insert_new_mark(&mut self, _set_idx: usize, _mark: (usize, usize)) {
        todo!()
    }

    pub fn clear_mark_set(&mut self, set_idx: usize) {
        self.mark_sets[set_idx].clear();
    }
}
