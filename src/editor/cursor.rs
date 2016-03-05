#![allow(dead_code)]

use std::slice::{Iter, IterMut};
use std::ops::{Index, IndexMut};
use std::cmp::Ordering;

use buffer::Buffer;
use formatter::LineFormatter;

/// A text cursor.  Also represents selections when range.0 != range.1.
///
/// `range` is a pair of 1d grapheme indexes into the text.
///
/// `vis_start` is the visual 2d horizontal position of the cursor.  This
/// doesn't affect editing operations at all, but is used for cursor movement.
#[derive(Copy, Clone)]
pub struct Cursor {
    pub range: (usize, usize), // start, end
    pub vis_start: usize, // start
}

impl Cursor {
    pub fn new() -> Cursor {
        Cursor {
            range: (0, 0),
            vis_start: 0,
        }
    }

    pub fn update_vis_start<T: LineFormatter>(&mut self, buf: &Buffer, f: &T) {
        self.vis_start = f.index_to_horizontal_v2d(buf, self.range.0);
    }
}


/// A collection of cursors, managed to always be in a consistent
/// state for multi-cursor editing.
pub struct CursorSet {
    cursors: Vec<Cursor>,
}


impl CursorSet {
    pub fn new() -> CursorSet {
        CursorSet { cursors: vec![Cursor::new()] }
    }

    pub fn add_cursor(&mut self, cursor: Cursor) {
        self.cursors.push(cursor);
        self.make_consistent();
    }

    pub fn truncate(&mut self, len: usize) {
        self.cursors.truncate(len);
    }

    pub fn iter<'a>(&'a self) -> Iter<'a, Cursor> {
        (&self.cursors[..]).iter()
    }

    pub fn iter_mut<'a>(&'a mut self) -> IterMut<'a, Cursor> {
        (&mut self.cursors[..]).iter_mut()
    }

    pub fn make_consistent(&mut self) {
        // First, sort the cursors by starting position
        self.cursors.sort_by(|a, b| {
            if a.range.0 < b.range.0 {
                Ordering::Less
            } else if a.range.0 > b.range.0 {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        });

        // Next, merge overlapping cursors
        let mut i = 0;
        while i < (self.cursors.len() - 1) {
            if self.cursors[i].range.1 >= self.cursors[i + 1].range.0 {
                self.cursors[i].range.1 = self.cursors[i + 1].range.1;
                self.cursors.remove(i + 1);
            } else {
                i += 1;
            }
        }
    }
}


impl Index<usize> for CursorSet {
    type Output = Cursor;

    fn index<'a>(&'a self, _index: usize) -> &'a Cursor {
        &(self.cursors[_index])
    }
}


impl IndexMut<usize> for CursorSet {
    fn index_mut<'a>(&'a mut self, _index: usize) -> &'a mut Cursor {
        &mut (self.cursors[_index])
    }
}
