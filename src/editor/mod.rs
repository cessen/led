#![allow(dead_code)]

use std::{
    cmp::{max, min},
    collections::HashMap,
    fs::File,
    io::{self, BufWriter, Write},
};

use backend::{
    buffer::{Buffer, BufferPath},
    marks::Mark,
};

use crate::{
    formatter::LineFormatter,
    graphemes::{
        is_grapheme_boundary, nth_next_grapheme_boundary, nth_prev_grapheme_boundary, RopeGraphemes,
    },
    string_utils::{rope_slice_to_line_ending, LineEnding},
    utils::digit_count,
};

pub struct Editor {
    pub buffer: Buffer,
    pub formatter: LineFormatter,
    pub line_ending_type: LineEnding,
    pub soft_tabs: bool,
    pub soft_tab_width: u8,

    // The dimensions of the total editor in screen space, including the
    // header, gutter, etc.
    pub editor_dim: (usize, usize),

    // The dimensions and position of just the text view portion of the editor
    pub view_dim: (usize, usize), // (height, width)

    // Indices into the mark sets of the buffer.
    pub v_msi: usize, // View position MarkSet index.
    pub c_msi: usize, // Cursors MarkSet index.
}

impl Editor {
    /// Create a new blank editor
    pub fn new(buffer: Buffer, formatter: LineFormatter) -> Editor {
        let mut buffer = buffer;

        // Create appropriate mark sets for view positions and cursors.
        let v_msi = buffer.add_mark_set();
        let c_msi = buffer.add_mark_set();
        buffer.mark_sets[v_msi].add_mark(Mark::new(0, 0));
        buffer.mark_sets[c_msi].add_mark(Mark::new(0, 0));

        let mut ed = Editor {
            buffer: buffer,
            formatter: formatter,
            line_ending_type: LineEnding::LF,
            soft_tabs: false,
            soft_tab_width: 4,
            editor_dim: (0, 0),
            view_dim: (0, 0),
            v_msi: v_msi,
            c_msi: c_msi,
        };

        ed.auto_detect_line_ending();
        ed.auto_detect_indentation_style();

        ed
    }

    pub fn save_if_dirty(&mut self) -> io::Result<()> {
        if let BufferPath::File(ref file_path) = self.buffer.path {
            if self.buffer.is_dirty {
                let mut f = BufWriter::new(File::create(file_path)?);

                for c in self.buffer.text.chunks() {
                    f.write(c.as_bytes())?;
                }

                self.buffer.is_dirty = false;
            }
        }

        Ok(())
    }

    pub fn auto_detect_line_ending(&mut self) {
        let mut line_ending_histogram: [usize; 8] = [0, 0, 0, 0, 0, 0, 0, 0];

        // Collect statistics on the first 100 lines
        for line in self.buffer.text.lines().take(100) {
            // Get the line ending
            let ending = if line.len_chars() == 1 {
                let g = RopeGraphemes::new(&line.slice((line.len_chars() - 1)..))
                    .last()
                    .unwrap();
                rope_slice_to_line_ending(&g)
            } else if line.len_chars() > 1 {
                let g = RopeGraphemes::new(&line.slice((line.len_chars() - 2)..))
                    .last()
                    .unwrap();
                rope_slice_to_line_ending(&g)
            } else {
                LineEnding::None
            };

            // Record which line ending it is
            match ending {
                LineEnding::None => {}
                LineEnding::CRLF => {
                    line_ending_histogram[0] += 1;
                }
                LineEnding::LF => {
                    line_ending_histogram[1] += 1;
                }
                LineEnding::VT => {
                    line_ending_histogram[2] += 1;
                }
                LineEnding::FF => {
                    line_ending_histogram[3] += 1;
                }
                LineEnding::CR => {
                    line_ending_histogram[4] += 1;
                }
                LineEnding::NEL => {
                    line_ending_histogram[5] += 1;
                }
                LineEnding::LS => {
                    line_ending_histogram[6] += 1;
                }
                LineEnding::PS => {
                    line_ending_histogram[7] += 1;
                }
            }
        }

        // Analyze stats and make a determination
        let mut lei = 0;
        let mut le_count = 0;
        for i in 0usize..8 {
            if line_ending_histogram[i] >= le_count {
                lei = i;
                le_count = line_ending_histogram[i];
            }
        }

        if le_count > 0 {
            self.line_ending_type = match lei {
                0 => LineEnding::CRLF,
                1 => LineEnding::LF,
                2 => LineEnding::VT,
                3 => LineEnding::FF,
                4 => LineEnding::CR,
                5 => LineEnding::NEL,
                6 => LineEnding::LS,
                7 => LineEnding::PS,

                _ => LineEnding::LF,
            };
        }
    }

    pub fn auto_detect_indentation_style(&mut self) {
        let mut tab_blocks: usize = 0;
        let mut space_blocks: usize = 0;
        let mut space_histogram: HashMap<usize, usize> = HashMap::new();

        let mut last_indent = (false, 0usize); // (was_tabs, indent_count)

        // Collect statistics on the first 1000 lines
        for line in self.buffer.text.lines().take(1000) {
            let mut c_iter = line.chars();
            match c_iter.next() {
                Some('\t') => {
                    // Count leading tabs
                    let mut count = 1;
                    for c in c_iter {
                        if c == '\t' {
                            count += 1;
                        } else {
                            break;
                        }
                    }

                    // Update stats
                    if last_indent.0 && last_indent.1 < count {
                        tab_blocks += 1;
                    }

                    // Store last line info
                    last_indent = (true, count);
                }

                Some(' ') => {
                    // Count leading spaces
                    let mut count = 1;
                    for c in c_iter {
                        if c == ' ' {
                            count += 1;
                        } else {
                            break;
                        }
                    }

                    // Update stats
                    if !last_indent.0 && last_indent.1 < count {
                        space_blocks += 1;
                        let amount = count - last_indent.1;
                        *space_histogram.entry(amount).or_insert(0) += 1;
                    }

                    // Store last line info
                    last_indent = (false, count);
                }

                _ => {}
            }
        }

        // Analyze stats and make a determination
        if space_blocks == 0 && tab_blocks == 0 {
            return;
        }

        if space_blocks > (tab_blocks * 2) {
            let mut width = 0;
            let mut width_count = 0;
            for (w, count) in space_histogram.iter() {
                if *count > width_count {
                    width = *w;
                    width_count = *count;
                }
            }

            self.soft_tabs = true;
            self.soft_tab_width = width as u8;
        } else {
            self.soft_tabs = false;
        }
    }

    /// Updates the view dimensions.
    pub fn update_dim(&mut self, h: usize, w: usize) {
        let line_count_digits = digit_count(self.buffer.text.len_lines() as u32, 10) as usize;
        self.editor_dim = (h, w);

        // Minus 1 vertically for the header, minus two more than the digits in
        // the line count for the gutter.
        self.view_dim = (
            self.editor_dim.0 - 1,
            self.editor_dim.1 - line_count_digits - 2,
        );
    }

    pub fn undo(&mut self) {
        // TODO: handle multiple cursors properly
        if let Some((_start, end)) = self.buffer.undo() {
            self.buffer.mark_sets[self.c_msi].reduce_to_main();
            self.buffer.mark_sets[self.c_msi][0].head = end;
            self.buffer.mark_sets[self.c_msi][0].tail = end;
            self.buffer.mark_sets[self.c_msi][0].hh_pos = None;

            self.move_view_to_cursor();
        }
    }

    pub fn redo(&mut self) {
        // TODO: handle multiple cursors properly
        if let Some((_start, end)) = self.buffer.redo() {
            self.buffer.mark_sets[self.c_msi].reduce_to_main();
            self.buffer.mark_sets[self.c_msi][0].head = end;
            self.buffer.mark_sets[self.c_msi][0].tail = end;
            self.buffer.mark_sets[self.c_msi][0].hh_pos = None;

            self.move_view_to_cursor();
        }
    }

    /// Moves the editor's view the minimum amount to show the cursor
    pub fn move_view_to_cursor(&mut self) {
        // Find the first and last char index visible within the editor.
        let c_first = self.formatter.set_horizontal(
            &self.buffer.text,
            self.buffer.mark_sets[self.v_msi][0].head,
            0,
        );
        let mut c_last = self.formatter.offset_vertical(
            &self.buffer.text,
            c_first,
            self.view_dim.0 as isize - 1,
        );
        c_last = self
            .formatter
            .set_horizontal(&self.buffer.text, c_last, self.view_dim.1);

        // Adjust the view depending on where the cursor is
        let cursor_head = self.buffer.mark_sets[self.c_msi].main().unwrap().head;
        if cursor_head < c_first {
            self.buffer.mark_sets[self.v_msi][0].head = cursor_head;
        } else if cursor_head > c_last {
            self.buffer.mark_sets[self.v_msi][0].head = self.formatter.offset_vertical(
                &self.buffer.text,
                cursor_head,
                -(self.view_dim.0 as isize),
            );
        }
    }

    pub fn insert_text_at_cursor(&mut self, text: &str) {
        // TODO: handle multiple cursors.
        let range = self.buffer.mark_sets[self.c_msi][0].range();

        // Do the edit.
        self.buffer.edit((range.start, range.end), text);

        // Adjust cursor position.
        let len = text.chars().count();
        self.buffer.mark_sets[self.c_msi][0].head = range.start + len;
        self.buffer.mark_sets[self.c_msi][0].tail = range.start + len;
        self.buffer.mark_sets[self.c_msi][0].hh_pos = None;

        // Adjust view
        self.move_view_to_cursor();
    }

    pub fn insert_tab_at_cursor(&mut self) {
        // TODO: handle multiple cursors.
        let range = self.buffer.mark_sets[self.c_msi][0].range();

        if self.soft_tabs {
            // Figure out how many spaces to insert
            let vis_pos = self
                .formatter
                .get_horizontal(&self.buffer.text, range.start);
            // TODO: handle tab settings
            let next_tab_stop =
                ((vis_pos / self.soft_tab_width as usize) + 1) * self.soft_tab_width as usize;
            let space_count = min(next_tab_stop - vis_pos, 8);

            // Insert spaces
            let space_strs = [
                "", " ", "  ", "   ", "    ", "     ", "      ", "       ", "        ",
            ];
            self.buffer
                .edit((range.start, range.end), space_strs[space_count]);

            // Adjust cursor position.
            self.buffer.mark_sets[self.c_msi][0].head = range.start + space_count;
            self.buffer.mark_sets[self.c_msi][0].tail = range.start + space_count;
            self.buffer.mark_sets[self.c_msi][0].hh_pos = None;
        } else {
            self.buffer.edit((range.start, range.end), "\t");

            // Adjust cursor position.
            self.buffer.mark_sets[self.c_msi][0].head = range.start + 1;
            self.buffer.mark_sets[self.c_msi][0].tail = range.start + 1;
            self.buffer.mark_sets[self.c_msi][0].hh_pos = None;
        }

        // Adjust view
        self.move_view_to_cursor();
    }

    pub fn remove_text_behind_cursor(&mut self, grapheme_count: usize) {
        // TODO: handle multiple cursors.
        let mark = self.buffer.mark_sets[self.c_msi].main().unwrap();
        let range = mark.range();

        // Do nothing if there's nothing to delete.
        if range.start == 0 {
            return;
        }

        let pre =
            nth_prev_grapheme_boundary(&self.buffer.text.slice(..), range.start, grapheme_count);

        // Remove text
        self.buffer.edit((pre, range.start), "");

        // Adjust view
        self.move_view_to_cursor();
    }

    pub fn remove_text_in_front_of_cursor(&mut self, grapheme_count: usize) {
        // TODO: handle multiple cursors.
        let mark = self.buffer.mark_sets[self.c_msi].main().unwrap();
        let range = mark.range();

        // Do nothing if there's nothing to delete.
        if range.end == self.buffer.text.len_chars() {
            return;
        }

        let post =
            nth_next_grapheme_boundary(&self.buffer.text.slice(..), range.end, grapheme_count);

        // Remove text
        self.buffer.edit((range.end, post), "");

        // Adjust view
        self.move_view_to_cursor();
    }

    pub fn remove_text_inside_cursor(&mut self) {
        // TODO: handle multiple cursors.
        let mark = self.buffer.mark_sets[self.c_msi].main().unwrap();
        let range = mark.range();

        if range.start < range.end {
            self.buffer.edit((range.start, range.end), "");
        }

        // Adjust view
        self.move_view_to_cursor();
    }

    pub fn cursor_to_beginning_of_buffer(&mut self) {
        self.buffer.mark_sets[self.c_msi].clear();
        self.buffer.mark_sets[self.c_msi].add_mark(Mark::new(0, 0));

        // Adjust view.
        self.move_view_to_cursor();
    }

    pub fn cursor_to_end_of_buffer(&mut self) {
        let end = self.buffer.text.len_chars();

        self.buffer.mark_sets[self.c_msi].clear();
        self.buffer.mark_sets[self.c_msi].add_mark(Mark::new(end, end));

        // Adjust view.
        self.move_view_to_cursor();
    }

    pub fn cursor_left(&mut self, n: usize) {
        for mark in self.buffer.mark_sets[self.c_msi].iter_mut() {
            mark.head = nth_prev_grapheme_boundary(&self.buffer.text.slice(..), mark.head, n);
            mark.tail = mark.head;
            mark.hh_pos = None;
        }
        self.buffer.mark_sets[self.c_msi].make_consistent();

        // Adjust view
        self.move_view_to_cursor();
    }

    pub fn cursor_right(&mut self, n: usize) {
        for mark in self.buffer.mark_sets[self.c_msi].iter_mut() {
            mark.head = nth_next_grapheme_boundary(&self.buffer.text.slice(..), mark.head, n);
            mark.tail = mark.head;
            mark.hh_pos = None;
        }
        self.buffer.mark_sets[self.c_msi].make_consistent();

        // Adjust view
        self.move_view_to_cursor();
    }

    pub fn cursor_up(&mut self, n: usize) {
        for mark in self.buffer.mark_sets[self.c_msi].iter_mut() {
            if mark.hh_pos == None {
                mark.hh_pos = Some(self.formatter.get_horizontal(&self.buffer.text, mark.head));
            }

            let vmove = -1 * n as isize;

            let mut temp_index =
                self.formatter
                    .offset_vertical(&self.buffer.text, mark.head, vmove);
            temp_index =
                self.formatter
                    .set_horizontal(&self.buffer.text, temp_index, mark.hh_pos.unwrap());

            if !is_grapheme_boundary(&self.buffer.text.slice(..), temp_index) {
                temp_index = nth_prev_grapheme_boundary(&self.buffer.text.slice(..), temp_index, 1);
            }

            if temp_index == mark.head {
                // We were already at the top.
                mark.head = 0;
                mark.tail = 0;
                mark.hh_pos = None;
            } else {
                mark.head = temp_index;
                mark.tail = temp_index;
            }
        }
        self.buffer.mark_sets[self.c_msi].make_consistent();

        // Adjust view
        self.move_view_to_cursor();
    }

    pub fn cursor_down(&mut self, n: usize) {
        for mark in self.buffer.mark_sets[self.c_msi].iter_mut() {
            if mark.hh_pos == None {
                mark.hh_pos = Some(self.formatter.get_horizontal(&self.buffer.text, mark.head));
            }

            let vmove = n as isize;

            let mut temp_index =
                self.formatter
                    .offset_vertical(&self.buffer.text, mark.head, vmove);
            temp_index =
                self.formatter
                    .set_horizontal(&self.buffer.text, temp_index, mark.hh_pos.unwrap());

            if !is_grapheme_boundary(&self.buffer.text.slice(..), temp_index) {
                temp_index = nth_prev_grapheme_boundary(&self.buffer.text.slice(..), temp_index, 1);
            }

            if temp_index == mark.head {
                // We were already at the bottom.
                mark.head = self.buffer.text.len_chars();
                mark.tail = self.buffer.text.len_chars();
                mark.hh_pos = None;
            } else {
                mark.head = temp_index;
                mark.tail = temp_index;
            }
        }
        self.buffer.mark_sets[self.c_msi].make_consistent();

        // Adjust view
        self.move_view_to_cursor();
    }

    pub fn page_up(&mut self) {
        let move_amount = self.view_dim.0 - max(self.view_dim.0 / 8, 1);
        self.buffer.mark_sets[self.v_msi][0].head = self.formatter.offset_vertical(
            &self.buffer.text,
            self.buffer.mark_sets[self.v_msi][0].head,
            -1 * move_amount as isize,
        );

        self.cursor_up(move_amount);

        // Adjust view
        self.move_view_to_cursor();
    }

    pub fn page_down(&mut self) {
        let move_amount = self.view_dim.0 - max(self.view_dim.0 / 8, 1);
        self.buffer.mark_sets[self.v_msi][0].head = self.formatter.offset_vertical(
            &self.buffer.text,
            self.buffer.mark_sets[self.v_msi][0].head,
            move_amount as isize,
        );

        self.cursor_down(move_amount);

        // Adjust view
        self.move_view_to_cursor();
    }

    pub fn jump_to_line(&mut self, n: usize) {
        self.buffer.mark_sets[self.c_msi].reduce_to_main();
        if self.buffer.mark_sets[self.c_msi][0].hh_pos == None {
            self.buffer.mark_sets[self.c_msi][0].hh_pos = Some(
                self.formatter
                    .get_horizontal(&self.buffer.text, self.buffer.mark_sets[self.c_msi][0].head),
            );
        }

        let pos = self.buffer.text.line_to_char(n);
        let pos = self.formatter.set_horizontal(
            &self.buffer.text,
            pos,
            self.buffer.mark_sets[self.c_msi][0].hh_pos.unwrap(),
        );

        self.buffer.mark_sets[self.c_msi][0].head = pos;
        self.buffer.mark_sets[self.c_msi][0].tail = pos;

        // Adjust view
        self.move_view_to_cursor();
    }
}
