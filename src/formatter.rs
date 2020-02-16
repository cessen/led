use std::borrow::Cow;

use ropey::{Rope, RopeSlice};

use crate::{
    buffer::Buffer,
    string_utils::char_count,
    string_utils::{is_line_ending, str_is_whitespace},
    utils::{grapheme_width, is_grapheme_boundary, prev_grapheme_boundary, RopeGraphemes},
};

// Maximum chars in a line before a soft line break is forced.
// This is necessary to prevent pathological formatting cases which
// could slow down the editor arbitrarily for arbitrarily long
// lines.
const LINE_BLOCK_LENGTH: usize = 1 << 12;

// A fudge-factor for the above block length, to allow looking for natural
// breaks.
const LINE_BLOCK_FUDGE: usize = 32;

//--------------------------------------------------------------------------

#[derive(Clone)]
pub struct LineFormatter {
    pub tab_width: usize,
    pub wrap_width: usize,
    pub maintain_indent: bool,
    pub wrap_extra_indent: usize,
}

impl LineFormatter {
    pub fn new(tab_width: usize) -> LineFormatter {
        LineFormatter {
            tab_width: tab_width,
            wrap_width: 80,
            maintain_indent: true,
            wrap_extra_indent: 2,
        }
    }

    /// Returns an iterator over the blocks of the buffer, starting at the
    /// block containing the given char.  Also returns the offset of that char
    /// relative to the start of the first block.
    pub fn iter<'b>(&'b self, buf: &'b Buffer, char_idx: usize) -> (Blocks<'b>, usize) {
        // Get the line.
        let (line_i, col_i) = buf.index_to_line_col(char_idx);
        let line = buf.get_line(line_i);

        // Find the right block in the line, and the index within that block
        let (block_index, block_range) = block_index_and_range(&line, col_i);
        let col_i_adjusted = col_i - block_range.0;

        (
            Blocks {
                formatter: self,
                buf: &buf.text,
                line_idx: line_i,
                line_block_count: block_count(&line),
                block_idx: block_index,
            },
            col_i_adjusted,
        )
    }

    /// Converts from char index to the horizontal 2d char index.
    pub fn get_horizontal(&self, buf: &Buffer, char_idx: usize) -> usize {
        let (_, vis_iter, char_offset) = self.block_vis_iter_and_char_offset(buf, char_idx);

        // Traverse the iterator and find the horizontal position of the char
        // index.
        let mut hpos = 0;
        let mut i = 0;
        let mut last_width = 0;

        for (g, pos, width) in vis_iter {
            hpos = pos.1;
            last_width = width;
            i += char_count(&g);

            if i > char_offset {
                return hpos;
            }
        }

        // If we went off the end, calculate the position of the end of the
        // block.
        return hpos + last_width;
    }

    /// Takes a char index and a desired visual horizontal position, and
    /// returns a char index on the same visual line as the given index,
    /// but offset to have the desired horizontal position (or as close as is
    /// possible.
    pub fn set_horizontal(&self, buf: &Buffer, char_idx: usize, horizontal: usize) -> usize {
        let (_, vis_iter, char_offset) = self.block_vis_iter_and_char_offset(buf, char_idx);

        let mut hpos_char_idx = None;
        let mut i = 0;
        let mut last_i = 0;
        let mut last_pos = (0, 0);
        for (g, pos, width) in vis_iter {
            // Check if we moved to the next line.
            if pos.0 > last_pos.0 {
                // If we did, but we're already passed the given char_idx,
                // that means the target was on the previous line but the line
                // wasn't long enough, so return the index of the last grapheme
                // of the previous line.
                if i > char_offset {
                    return last_i;
                }

                // Otherwise reset and keep going.
                hpos_char_idx = None;
            }

            // Check if we found the horizontal position on this line,
            // and set it if so.
            if hpos_char_idx == None && horizontal < (pos.1 + width) {
                hpos_char_idx = Some(i);
            }

            // Check if we've found the horizontal position _and_ the passed
            // char_idx on the same line, and return if so.
            if i >= char_offset && hpos_char_idx != None {
                return hpos_char_idx.unwrap();
            }

            last_pos = pos;
            last_i = i;
            i += char_count(&g);
        }

        // If we reached the end of the text, return the last char index.
        return i;
    }

    /// Takes a char index and a visual vertical offset, and returns the char
    /// index after that visual offset is applied.
    pub fn offset_vertical(&self, buf: &Buffer, char_idx: usize, v_offset: isize) -> usize {
        let mut char_idx = char_idx;
        let mut v_offset = v_offset;
        while v_offset != 0 {
            // Get our block and the offset of the char inside it.
            let (block, block_vis_iter, char_offset) =
                self.block_vis_iter_and_char_offset(buf, char_idx);

            // Get the vertical size of the block and the vertical
            // position of the char_idx within it.
            let block_v_dim = block_vis_iter.clone().last().map(|n| (n.1).0).unwrap_or(0) + 1;
            let char_v_pos = block_vis_iter.clone().vpos(char_offset);

            // Get the char's vertical position within the block after offset
            // by v_offset.
            let offset_char_v_pos = char_v_pos as isize + v_offset;

            // Check if the offset position is within the block or not,
            // and handle appropriately.
            if offset_char_v_pos < 0 {
                // If we're off the start of the block.
                if char_idx == 0 {
                    // We reached the start of the whole buffer.
                    break;
                } else {
                    // Set our variables appropriately for the next iteration.
                    char_idx -= char_offset + 1;
                    v_offset += char_v_pos as isize + 1;
                }
            } else if offset_char_v_pos >= block_v_dim as isize {
                // If we're off the end of the block.
                if char_idx >= buf.text.len_chars() {
                    // We reached the end of the whole buffer.
                    char_idx = buf.text.len_chars();
                    break;
                } else {
                    // Set our variables appropriately for the next iteration.
                    char_idx += block.len_chars() - char_offset;
                    v_offset -= block_v_dim as isize - char_v_pos as isize;
                }
            } else {
                // If the vertical offset is within this block, calculate an
                // appropriate char index and return.
                let mut i = 0;
                for (g, pos, _) in block_vis_iter {
                    if pos.0 == offset_char_v_pos as usize {
                        break;
                    }
                    i += char_count(&g);
                }
                char_idx += block.len_chars() - char_offset + i;
                v_offset = 0;
            }
        }

        return char_idx;
    }

    //----------------------------------------------------
    // Helper methods

    /// Returns the amount of indentation to use for soft-line wrapping
    /// given the start of a line.
    fn get_line_indent(&self, line: &RopeSlice) -> usize {
        if !self.maintain_indent {
            return 0;
        }

        let mut indent = 0;
        for c in line.chars() {
            match c {
                ' ' => {
                    indent += 1;
                }
                '\t' => {
                    indent = tab_stop_from_vis_pos(indent, self.tab_width);
                }
                _ => break,
            }

            // If the indent is too long for the wrap width, do no indentation.
            if (indent + self.wrap_extra_indent + 2) > self.wrap_width {
                return 0;
            }
        }

        indent
    }

    /// Returns the appropriate BlockVisIter containing the given char, and the
    /// char's offset within that iter.
    fn block_vis_iter_and_char_offset<'b>(
        &self,
        buf: &'b Buffer,
        char_idx: usize,
    ) -> (RopeSlice<'b>, BlockVisIter<'b>, usize) {
        let (line_i, col_i) = buf.index_to_line_col(char_idx);
        let line = buf.get_line(line_i);

        // Find the right block in the line, and the index within that block
        let (block_index, block_range) = block_index_and_range(&line, col_i);
        let col_i_adjusted = col_i - block_range.0;

        // Get the right block and an iter into it.
        let block = line.slice(block_range.0..block_range.1);
        let g_iter = RopeGraphemes::new(&block);

        // Get an appropriate visual block iter.
        let vis_iter = BlockVisIter::new(
            g_iter,
            self.wrap_width,
            self.tab_width,
            block_index == 0,
            if block_index == 0 {
                0
            } else {
                self.get_line_indent(&line)
            },
            self.wrap_extra_indent,
        );

        (block, vis_iter, col_i_adjusted)
    }
}

//--------------------------------------------------------------------------

#[derive(Clone)]
pub struct Blocks<'a> {
    formatter: &'a LineFormatter,
    buf: &'a Rope,
    line_idx: usize,
    line_block_count: usize,
    block_idx: usize,
}

impl<'a> Iterator for Blocks<'a> {
    type Item = (BlockVisIter<'a>, bool);

    fn next(&mut self) -> Option<Self::Item> {
        // Check if we're done already.
        if self.line_idx >= self.buf.len_lines() {
            return None;
        }

        // Get our return values.
        let (iter, is_line_start) = {
            let line = self.buf.line(self.line_idx);
            let (start, end) = char_range_from_block_index(&line, self.block_idx);
            let block = line.slice(start..end);
            let iter = BlockVisIter::new(
                RopeGraphemes::new(&block),
                self.formatter.wrap_width,
                self.formatter.tab_width,
                self.block_idx == 0,
                if self.block_idx == 0 {
                    0
                } else {
                    self.formatter.get_line_indent(&line)
                },
                self.formatter.wrap_extra_indent,
            );

            (iter, self.block_idx == 0)
        };

        // Progress the values of the iterator.
        self.block_idx += 1;
        if self.block_idx >= self.line_block_count {
            self.line_idx += 1;
            self.block_idx = 0;
            if self.line_idx < self.buf.len_lines() {
                self.line_block_count = block_count(&self.buf.line(self.line_idx));
            }
        }

        // Return.
        Some((iter, is_line_start))
    }
}

//--------------------------------------------------------------------------

/// An iterator over the visual printable characters of a block of text,
/// yielding the text of the character, its position in 2d space, and its
/// visial width.
#[derive(Clone)]
pub struct BlockVisIter<'a> {
    grapheme_itr: RopeGraphemes<'a>,

    wrap_width: usize,
    tab_width: usize,
    indent: usize,            // Size of soft indent to use.
    wrap_extra_indent: usize, // Additional amount to indent soft-wrapped lines.
    finding_indent: bool,

    word_buf: Vec<(Cow<'a, str>, usize)>, // Printable character and its width.
    word_i: usize,
    pos: (usize, usize),
}

impl<'a> BlockVisIter<'a> {
    fn new(
        grapheme_itr: RopeGraphemes<'a>,
        wrap_width: usize,
        tab_width: usize,
        find_indent: bool,
        starting_indent: usize,
        wrap_extra_indent: usize,
    ) -> BlockVisIter<'a> {
        BlockVisIter {
            grapheme_itr: grapheme_itr,
            wrap_width: wrap_width,
            tab_width: tab_width,
            indent: starting_indent,
            wrap_extra_indent: wrap_extra_indent,
            finding_indent: find_indent,

            word_buf: Vec::new(),
            word_i: 0,
            pos: (0, 0),
        }
    }

    pub fn vpos(&mut self, char_offset: usize) -> usize {
        let mut vpos = 0;
        let mut i = 0;

        for (g, pos, _) in self {
            vpos = pos.0;
            i += char_count(&g);

            if i > char_offset {
                break;
            }
        }

        vpos
    }
}

impl<'a> Iterator for BlockVisIter<'a> {
    type Item = (Cow<'a, str>, (usize, usize), usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos == (0, 0) {
            self.pos = (0, self.indent);
        }

        // Get next word if necessary
        if self.word_i >= self.word_buf.len() {
            let mut word_width = 0;
            self.word_buf.truncate(0);

            while let Some(g) = self.grapheme_itr.next().map(|g| Cow::<str>::from(g)) {
                let width =
                    grapheme_vis_width_at_vis_pos(&g, self.pos.1 + word_width, self.tab_width);
                self.word_buf.push((g.clone(), width));
                word_width += width;

                if str_is_whitespace(&g) {
                    if self.finding_indent && (g.as_bytes()[0] == 0x09 || g.as_bytes()[0] == 0x20) {
                        if (self.indent + self.wrap_extra_indent + width + 2) > self.wrap_width {
                            // Cancel indentation if it's too long for the screen.
                            self.indent = 0;
                            self.wrap_extra_indent = 0;
                            self.finding_indent = false;
                        } else {
                            self.indent += width;
                        }
                    }
                    break;
                } else {
                    self.finding_indent = false;
                }
            }

            if self.word_buf.len() == 0 {
                return None;
            }

            // Move to next line if necessary
            if (self.pos.1 + word_width) > self.wrap_width && (self.pos.1 > self.indent) {
                if self.pos.1 > 0 {
                    self.pos = (self.pos.0 + 1, self.indent + self.wrap_extra_indent);
                }
            }

            self.word_i = 0;
        }

        // Get next grapheme and width from the current word.
        let (g, g_width) = {
            let (ref g, mut width) = self.word_buf[self.word_i];
            if g == "\t" {
                width = grapheme_vis_width_at_vis_pos(&g, self.pos.1, self.tab_width);
            }
            (g, width)
        };

        // Get our character's position and update the position for the next
        // grapheme.
        if (self.pos.1 + g_width) > self.wrap_width && self.pos.1 > 0 {
            self.pos.0 += 1;
            self.pos.1 = self.indent + self.wrap_extra_indent;
        }
        let pos = self.pos;
        self.pos.1 += g_width;

        // Increment index and return.
        self.word_i += 1;
        return Some((g.clone(), pos, g_width));
    }
}

/// Returns the visual width of a grapheme given a starting
/// position on a line.
fn grapheme_vis_width_at_vis_pos(g: &str, pos: usize, tab_width: usize) -> usize {
    if g == "\t" {
        return tab_stop_from_vis_pos(pos, tab_width) - pos;
    } else if is_line_ending(g) {
        return 1;
    } else {
        return grapheme_width(&g);
    }
}

fn tab_stop_from_vis_pos(pos: usize, tab_width: usize) -> usize {
    ((pos / tab_width) + 1) * tab_width
}

//--------------------------------------------------------------------------

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
