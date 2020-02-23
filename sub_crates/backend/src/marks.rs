/// A mark on a piece of text, useful for representing cursors, selections, and
/// general positions within a piece of text.
///
/// Both the head and tail are specified in absolute positions from the start
/// of the text, and can have any relative relationship to each other (i.e. the
/// tail can be before, after, or at the same position as the head).
///
/// For cursors and selections, the `head` should be considered the part that is
/// moved by the user when e.g. extending selections.
///
/// `hh_pos` represents a target visual horizontal position of the mark's head,
/// and is useful when e.g. moving cursors up/down vertically in the text.  But
/// it can be ignored most of the time, does not effect editing operations in
/// any way, and is frequently set to `None`.
#[derive(Debug, Copy, Clone)]
pub struct Mark {
    pub head: usize,
    pub tail: usize,
    pub hh_pos: Option<usize>,
}

impl Mark {
    pub fn new(head: usize, tail: usize) -> Mark {
        Mark {
            head: head,
            tail: tail,
            hh_pos: None,
        }
    }

    /// Returns the properly sorted range of the mark.
    pub fn range(&self) -> std::ops::Range<usize> {
        if self.head < self.tail {
            std::ops::Range::<usize> {
                start: self.head,
                end: self.tail,
            }
        } else {
            std::ops::Range::<usize> {
                start: self.tail,
                end: self.head,
            }
        }
    }

    #[must_use]
    pub fn merge(&self, other: Mark) -> Mark {
        let r1 = self.range();
        let r2 = other.range();

        let r3 = (r1.start.min(r2.start), r1.end.max(r2.end));

        if self.head < self.tail {
            Mark {
                head: r3.0,
                tail: r3.1,
                hh_pos: None,
            }
        } else {
            Mark {
                head: r3.1,
                tail: r3.0,
                hh_pos: None,
            }
        }
    }

    /// Modify the mark based on an edit that occured to the text.
    ///
    /// `range` is the char range affected by the edit, and `new_len` is the
    /// new length of that range after the edit.
    ///
    /// `range` must be correctly ordered.
    #[must_use]
    pub fn edit(&self, range: (usize, usize), new_len: usize) -> Mark {
        assert!(range.0 <= range.1);

        // Head.
        let head = if self.head > range.1 {
            self.head + new_len - (range.1 - range.0)
        } else if self.head >= range.0 {
            range.0 + new_len
        } else {
            self.head
        };

        // Tail.
        let tail = if self.tail > range.1 {
            self.tail + new_len - (range.1 - range.0)
        } else if self.tail >= range.0 {
            range.0 + new_len
        } else {
            self.tail
        };

        Mark {
            head: head,
            tail: tail,
            hh_pos: None,
        }
    }
}

//----------------------------------------------------------------------

/// A set of disjoint Marks, sorted by position in the text.
///
/// Because the `marks` `Vec` in this struct is publicly exposed, we can't
/// actually guarantee that the marks are disjoint and sorted at all times, so
/// do not rely on that for safety.  However, these invariants *should* hold,
/// and code that modifies a MarkSet should ensure that the invariants remain
/// true.
///
/// The `make_consistent` method will ensure that all expected invariants hold,
/// modifying the set to meet the invariants if needed.
#[derive(Debug, Clone)]
pub struct MarkSet {
    pub main_mark_idx: usize,
    pub marks: Vec<Mark>,
}

impl MarkSet {
    /// Creates an empty MarkSet.
    pub fn new() -> MarkSet {
        MarkSet {
            main_mark_idx: 0,
            marks: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.main_mark_idx = 0;
        self.marks.clear();
    }

    pub fn truncate(&mut self, len: usize) {
        self.marks.truncate(len);
        self.main_mark_idx = self.main_mark_idx.min(self.marks.len().saturating_sub(1));
    }

    /// Returns the main mark, if it exists.
    pub fn main(&self) -> Option<Mark> {
        self.marks.get(self.main_mark_idx).map(|m| *m)
    }

    /// Removes all marks except the main one.
    pub fn reduce_to_main(&mut self) {
        self.marks.swap(0, self.main_mark_idx);
        self.main_mark_idx = 0;
        self.marks.truncate(1);
    }

    /// Adds a new mark to the set, inserting it into its sorted position, and
    /// returns the index where it was inserted.
    ///
    /// This assumes that all marks are already sorted by the start of their
    /// range.
    ///
    /// This does *not* preserve disjointedness.  You should call
    /// `make_consistent` after you have added all the marks you want.
    ///
    /// Runs in O(N + log N) time worst-case, but when the new mark is
    /// inserted at the end of the set it is amortized O(1).
    pub fn add_mark(&mut self, mark: Mark) -> usize {
        // Special case optimization and early-out for inserting at the end of
        // the set.
        if self
            .marks
            .last()
            .map(|l| l.range().start < mark.range().start)
            .unwrap_or(true)
        {
            self.marks.push(mark);
            return self.marks.len() - 1;
        }

        // Insert the mark.
        let idx = self
            .marks
            .binary_search_by_key(&mark.range().start, |m| m.range().start)
            .unwrap_or_else(|e| e);
        self.marks.insert(idx, mark);

        // Update the main_mark_idx.
        if self.main_mark_idx >= idx && self.marks.len() > 1 {
            self.main_mark_idx += 1;
        }

        idx
    }

    /// Merges all marks that are non-disjoint or not sorted relative to each
    /// other.
    ///
    /// This results in a set of fully sorted, disjoint marks.  If the set
    /// is already sorted and disjoint, no modifications are made.
    ///
    /// Note that even though the result is a sorted set, this _does not_ sort
    /// the set in any expected way.  Rather, it merges marks that aren't
    /// sorted.  For example, if the first mark is at the end of the text, and
    /// the last mark is at the start of the text, all marks will be merged
    /// into one.
    ///
    /// Runs in O(N) time.
    pub fn make_consistent(&mut self) {
        let mut i1 = 0;
        let mut i2 = 1;

        while i2 < self.marks.len() {
            if self.marks[i1].range().end < self.marks[i2].range().start {
                i1 += 1;
                self.marks[i1] = self.marks[i2];
                if self.main_mark_idx == i2 {
                    self.main_mark_idx = i1;
                }
                i2 += 1;
            } else {
                self.marks[i1] = self.marks[i1].merge(self.marks[i2]);
                if self.main_mark_idx == i2 {
                    self.main_mark_idx = i1;
                }
                i2 += 1;
            }
        }

        self.marks.truncate(i1 + 1);
    }

    pub fn iter(&self) -> std::slice::Iter<Mark> {
        self.marks.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<Mark> {
        self.marks.iter_mut()
    }
}

impl std::ops::Index<usize> for MarkSet {
    type Output = Mark;

    fn index(&self, index: usize) -> &Mark {
        &self.marks[index]
    }
}

impl std::ops::IndexMut<usize> for MarkSet {
    fn index_mut(&mut self, index: usize) -> &mut Mark {
        &mut self.marks[index]
    }
}
