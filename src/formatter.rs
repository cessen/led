use std::cmp::min;

use ropey::RopeSlice;

use crate::{
    buffer::Buffer,
    utils::{is_grapheme_boundary, prev_grapheme_boundary, RopeGraphemes},
};

// Maximum chars in a line before a soft line break is forced.
// This is necessary to prevent pathological formatting cases which
// could slow down the editor arbitrarily for arbitrarily long
// lines.
const LINE_BLOCK_LENGTH: usize = 1 << 12;

// A fudge-factor for the above block length, to allow looking for natural
// breaks.
const LINE_BLOCK_FUDGE: usize = 32;

#[allow(dead_code)]
#[derive(Copy, Clone, PartialEq)]
pub enum RoundingBehavior {
    Round,
    Floor,
    Ceiling,
}

pub trait LineFormatter {
    /// Returns the 2d visual dimensions of the given text when formatted
    /// by the formatter.
    /// The text to be formatted is passed as a grapheme iterator.
    fn dimensions<'a, T>(&'a self, g_iter: T) -> (usize, usize)
    where
        T: Iterator<Item = RopeSlice<'a>>;

    /// Converts a char index within a text into a visual 2d position.
    /// The text to be formatted is passed as a grapheme iterator.
    fn index_to_v2d<'a, T>(&'a self, g_iter: T, char_idx: usize) -> (usize, usize)
    where
        T: Iterator<Item = RopeSlice<'a>>;

    /// Converts a visual 2d position into a char index within a text.
    /// The text to be formatted is passed as a grapheme iterator.
    fn v2d_to_index<'a, T>(
        &'a self,
        g_iter: T,
        v2d: (usize, usize),
        rounding: (RoundingBehavior, RoundingBehavior),
    ) -> usize
    where
        T: Iterator<Item = RopeSlice<'a>>;

    /// Converts from char index to the horizontal 2d char index.
    fn index_to_horizontal_v2d(&self, buf: &Buffer, char_idx: usize) -> usize {
        let (line_i, col_i) = buf.index_to_line_col(char_idx);
        let line = buf.get_line(line_i);

        // Find the right block in the line, and the index within that block
        let (_, block_range) = block_index_and_range(&line, col_i);
        let col_i_adjusted = col_i - block_range.0;

        // Get an iter into the right block
        let g_iter = RopeGraphemes::new(&line.slice(block_range.0..block_range.1));
        return self.index_to_v2d(g_iter, col_i_adjusted).1;
    }

    /// Takes a char index and a visual vertical offset, and returns the char
    /// index after that visual offset is applied.
    fn index_offset_vertical_v2d(
        &self,
        buf: &Buffer,
        char_idx: usize,
        offset: isize,
        rounding: (RoundingBehavior, RoundingBehavior),
    ) -> usize {
        // TODO: handle rounding modes
        // TODO: do this with bidirectional line iterator

        // Get the line and char index within that line.
        let (mut line_i, mut col_i) = buf.index_to_line_col(char_idx);
        let mut line = buf.get_line(line_i);

        // Get the block information for the char offset in the line.
        let (line_block, block_range) = block_index_and_range(&line, col_i);
        let col_i_adjusted = col_i - block_range.0;

        // Get the 2d coordinates within the block.
        let (mut y, x) = self.index_to_v2d(
            RopeGraphemes::new(&line.slice(block_range.0..block_range.1)),
            col_i_adjusted,
        );

        // First, find the right line while keeping track of the vertical offset
        let mut new_y = y as isize + offset;

        let mut block_index: usize = line_block;
        loop {
            line = buf.get_line(line_i);
            let (block_start, block_end) = char_range_from_block_index(&line, block_index);
            let (h, _) = self.dimensions(RopeGraphemes::new(&line.slice(block_start..block_end)));

            if new_y >= 0 && new_y < h as isize {
                y = new_y as usize;
                break;
            } else {
                if new_y > 0 {
                    let is_last_block = block_index >= (block_count(&line) - 1);

                    // Check for off-the-end
                    if is_last_block && (line_i + 1) >= buf.line_count() {
                        return buf.char_count();
                    }

                    if is_last_block {
                        line_i += 1;
                        block_index = 0;
                    } else {
                        block_index += 1;
                    }
                    new_y -= h as isize;
                } else if new_y < 0 {
                    // Check for off-the-end
                    if block_index == 0 && line_i == 0 {
                        return 0;
                    }

                    if block_index == 0 {
                        line_i -= 1;
                        line = buf.get_line(line_i);
                        block_index = block_count(&line) - 1;
                    } else {
                        block_index -= 1;
                    }
                    let (block_start, block_end) = char_range_from_block_index(&line, block_index);
                    let (h, _) =
                        self.dimensions(RopeGraphemes::new(&line.slice(block_start..block_end)));
                    new_y += h as isize;
                } else {
                    unreachable!();
                }
            }
        }

        // Next, convert the resulting coordinates back into buffer-wide
        // coordinates.
        let (block_start, block_end) = char_range_from_block_index(&line, block_index);
        let block_len = block_end - block_start;
        let block_slice = line.slice(block_start..block_end);
        let block_col_i = min(
            self.v2d_to_index(RopeGraphemes::new(&block_slice), (y, x), rounding),
            block_len.saturating_sub(1),
        );
        col_i = block_start + block_col_i;

        return buf.line_col_to_index((line_i, col_i));
    }

    /// Takes a char index and a desired visual horizontal position, and
    /// returns a char index on the same visual line as the given index,
    /// but offset to have the desired horizontal position.
    fn index_set_horizontal_v2d(
        &self,
        buf: &Buffer,
        char_idx: usize,
        horizontal: usize,
        rounding: RoundingBehavior,
    ) -> usize {
        // Get the line info.
        let (line_i, col_i) = buf.index_to_line_col(char_idx);
        let line = buf.get_line(line_i);

        // Get the right block within the line.
        let (block_i, block_range) = block_index_and_range(&line, col_i);
        let col_i_adjusted = col_i - block_range.0;

        // Calculate the horizontal position.
        let (v, _) = self.index_to_v2d(
            RopeGraphemes::new(&line.slice(block_range.0..block_range.1)),
            col_i_adjusted,
        );
        let block_col_i = self.v2d_to_index(
            RopeGraphemes::new(&line.slice(block_range.0..block_range.1)),
            (v, horizontal),
            (RoundingBehavior::Floor, rounding),
        );
        let new_col_i = if (line_i + 1) < buf.line_count() || (block_i + 1) < block_count(&line) {
            min(block_range.0 + block_col_i, block_range.1.saturating_sub(1))
        } else {
            min(block_range.0 + block_col_i, block_range.1)
        };

        return (char_idx + new_col_i) - col_i;
    }
}

// Finds the best break at or before the given char index, bounded by
// the given `lower_limit`.
pub fn find_good_break(slice: &RopeSlice, lower_limit: usize, char_idx: usize) -> usize {
    const WS_CHARS: &[char] = &[' ', '　', '\t'];

    let slice_len = slice.len_chars();
    let char_idx = char_idx.min(slice_len);
    let lower_limit = lower_limit.min(slice_len);

    // Early out in trivial cases.
    if char_idx < (LINE_BLOCK_LENGTH - LINE_BLOCK_FUDGE) {
        return char_idx;
    }

    // Find a whitespace break, if any.
    let mut i = char_idx;
    let mut prev = if i == slice_len {
        None
    } else {
        Some(slice.char(char_idx))
    };
    let mut char_itr = slice.chars_at(char_idx);
    while i > lower_limit {
        let c = char_itr.prev();
        if WS_CHARS.contains(&c.unwrap()) && prev.map(|pc| !WS_CHARS.contains(&pc)).unwrap_or(true)
        {
            return i;
        }
        prev = c;
        i -= 1;
    }

    // Otherwise, at least try to find a grapheme break.
    if is_grapheme_boundary(slice, char_idx) {
        char_idx
    } else {
        let i = prev_grapheme_boundary(slice, char_idx);
        if i > lower_limit {
            i
        } else {
            char_idx
        }
    }
}

pub fn char_range_from_block_index(slice: &RopeSlice, block_idx: usize) -> (usize, usize) {
    let start = {
        let initial = LINE_BLOCK_LENGTH * block_idx;
        find_good_break(slice, initial.saturating_sub(LINE_BLOCK_FUDGE), initial)
    };

    let end = {
        let initial = LINE_BLOCK_LENGTH * (block_idx + 1);
        find_good_break(slice, initial.saturating_sub(LINE_BLOCK_FUDGE), initial)
    };

    (start, end)
}

pub fn block_index_and_range(slice: &RopeSlice, char_idx: usize) -> (usize, (usize, usize)) {
    let mut block_index = char_idx / LINE_BLOCK_LENGTH;
    let mut range = char_range_from_block_index(slice, block_index);
    if char_idx >= range.1 && range.1 < slice.len_chars() {
        block_index += 1;
        range = char_range_from_block_index(slice, block_index);
    }
    (block_index, range)
}

pub fn block_count(slice: &RopeSlice) -> usize {
    let char_count = slice.len_chars();
    let mut last_idx = char_count.saturating_sub(1) / LINE_BLOCK_LENGTH;

    let range = char_range_from_block_index(slice, last_idx + 1);
    if range.0 < range.1 {
        last_idx += 1;
    }

    last_idx + 1
}
