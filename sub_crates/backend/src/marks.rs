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
}

//--------------------------------------------------------------------------

/// A set of disjoint Marks, sorted by position in the text.
///
/// Because the `marks` `Vec` in this struct is publicly exposed, we can't
/// actually guarantee that the marks are disjoint and sorted at all times, so
/// do not rely on that for safety.  However, these invariants *should* hold,
/// and code that modifies a MarkSet should ensure that the invariants remain
/// true.
///
/// The `merge_touching` method will ensure that all expected invariants hold,
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

    /// Adds a new mark to the set, inserting it into its sorted position, and
    /// returns the index where it was inserted.
    ///
    /// This assumes that all marks are already sorted by the start of their
    /// range.
    ///
    /// This does *not* preserve disjointedness.  You should call
    /// `merge_touching` after you have added all the marks you want.
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
    pub fn merge_touching(&mut self) {
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
}
