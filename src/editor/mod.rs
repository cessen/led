#![allow(dead_code)]

use buffer::Buffer;
use std::path::Path;
use std::cmp::min;
use files::{load_file_to_buffer, save_buffer_to_file};
use string_utils::grapheme_count;
use self::cursor::{Cursor, CursorSet};

mod cursor;


pub struct Editor {
    pub buffer: Buffer,
    pub file_path: Path,
    pub soft_tabs: bool,
    pub dirty: bool,
    
    // The dimensions and position of the editor's view within the buffer
    pub view_dim: (usize, usize),  // (height, width)
    pub view_pos: (usize, usize),  // (line, col)
    
    // The editing cursor position
    pub cursors: CursorSet,  
}


impl Editor {
    /// Create a new blank editor
    pub fn new() -> Editor {
        Editor {
            buffer: Buffer::new(),
            file_path: Path::new(""),
            soft_tabs: false,
            dirty: false,
            view_dim: (0, 0),
            view_pos: (0, 0),
            cursors: CursorSet::new(),
        }
    }
    
    pub fn new_from_file(path: &Path) -> Editor {
        let buf = match load_file_to_buffer(path) {
            Ok(b) => {b},
            _ => {Buffer::new()}
        };
        
        let mut ed = Editor {
            buffer: buf,
            file_path: path.clone(),
            soft_tabs: false,
            dirty: false,
            view_dim: (0, 0),
            view_pos: (0, 0),
            cursors: CursorSet::new(),
        };
        
        // For multiple-cursor testing
        let mut cur = Cursor::new();
        cur.range.0 = 30;
        cur.range.1 = 30;
        cur.update_vis_start(&(ed.buffer));
        ed.cursors.add_cursor(cur);
        
        ed.auto_detect_indentation_style();
        
        return ed;
    }
    
    pub fn save_if_dirty(&mut self) {
        if self.dirty && self.file_path != Path::new("") {
            let _ = save_buffer_to_file(&self.buffer, &self.file_path);
            self.dirty = false;
        }
    }
    
    pub fn auto_detect_indentation_style(&mut self) {
        let mut tab_blocks: usize = 0;
        let mut space_blocks: usize = 0;
        let mut space_histogram: [usize; 9] = [0, 0, 0, 0, 0, 0, 0, 0, 0];
        
        let mut last_indent = (false, 0us);  // (was_tabs, indent_count)
        
        // Collect statistics
        let mut line_i: usize = 0;
        for line in self.buffer.line_iter() {
            let mut g_iter = line.grapheme_iter();
            match g_iter.next() {
                Some("\t") => {
                    // Count leading tabs
                    let mut count = 1;
                    for g in g_iter {
                        if g == "\t" {
                            count += 1;
                        }
                        else {
                            break;
                        }
                    }
                    
                    // Update stats
                    if last_indent.0 && last_indent.1 < count {
                        tab_blocks += 1;
                    }
                    
                    // Store last line info
                    last_indent = (true, count);
                },
                
                Some(" ") => {
                    // Count leading spaces
                    let mut count = 1;
                    for g in g_iter {
                        if g == " " {
                            count += 1;
                        }
                        else {
                            break;
                        }
                    }
                    
                    // Update stats
                    if !last_indent.0 && last_indent.1 < count {
                        space_blocks += 1;
                        let amount = count - last_indent.1;
                        if amount < 9 {
                            space_histogram[amount] += 1;
                        }
                        else {
                            space_histogram[8] += 1;
                        }
                    }
                    
                    // Store last line info
                    last_indent = (false, count);
                },
                
                _ => {},
            }
            
            // Stop after 1000 lines
            line_i += 1;
            if line_i > 1000 {
                break;
            }
        }
        
        // Analyze stats and make a determination
        if space_blocks == 0 && tab_blocks == 0 {
            return;
        }
        
        if space_blocks > (tab_blocks * 2) {
            let mut width = 0;
            let mut width_count = 0;
            for i in range(0, 9) {
                if space_histogram[i] > width_count {
                    width = i;
                    width_count = space_histogram[i];
                }
            }
            
            self.soft_tabs = true;
            self.buffer.tab_width = width;
        }
        else {
            self.soft_tabs = false;
        }
    }
    
    pub fn update_dim(&mut self, h: usize, w: usize) {
        self.view_dim = (h, w);
    }
    
    
    pub fn undo(&mut self) {
        // TODO: handle multiple cursors properly
        if let Some(pos) = self.buffer.undo() {
            self.cursors.truncate(1);
            self.cursors[0].range.0 = pos;
            self.cursors[0].range.1 = pos;
            self.cursors[0].update_vis_start(&(self.buffer));
            
            self.move_view_to_cursor();
            
            self.dirty = true;
            
            self.cursors.make_consistent();
        }
    }
    
    
    pub fn redo(&mut self) {
        // TODO: handle multiple cursors properly
        if let Some(pos) = self.buffer.redo() {
            self.cursors.truncate(1);
            self.cursors[0].range.0 = pos;
            self.cursors[0].range.1 = pos;
            self.cursors[0].update_vis_start(&(self.buffer));
            
            self.move_view_to_cursor();
            
            self.dirty = true;
            
            self.cursors.make_consistent();
        }
    }
    
    
    /// Moves the editor's view the minimum amount to show the cursor
    pub fn move_view_to_cursor(&mut self) {
        // TODO: handle multiple cursors properly.  Should only move if
        // there are no cursors currently in view, and should jump to
        // the closest cursor.
        let (v, h) = self.buffer.index_to_v2d(self.cursors[0].range.0);
        
        // Horizontal
        if h < self.view_pos.1 {
            self.view_pos.1 = h;
        }
        else if h >= (self.view_pos.1 + self.view_dim.1) {
            self.view_pos.1 = 1 + h - self.view_dim.1;
        }
        
        // Vertical
        if v < self.view_pos.0 {
            self.view_pos.0 = v;
        }
        else if v >= (self.view_pos.0 + self.view_dim.0) {
            self.view_pos.0 = 1 + v - self.view_dim.0;
        }
    }
    
    pub fn insert_text_at_cursor(&mut self, text: &str) {
        self.cursors.make_consistent();
        
        let str_len = grapheme_count(text);
        let mut offset = 0;
        
        for c in self.cursors.iter_mut() {
            // Insert text
            self.buffer.insert_text(text, c.range.0 + offset);
            self.dirty = true;
            
            // Move cursor
            c.range.0 += str_len + offset;
            c.range.1 += str_len + offset;
            c.update_vis_start(&(self.buffer));
            
            // Update offset
            offset += str_len;
        }
        
        // Adjust view
        self.move_view_to_cursor();
    }
    
    pub fn insert_tab_at_cursor(&mut self) {
        self.cursors.make_consistent();
        
        if self.soft_tabs {
            let mut offset = 0;
            
            for c in self.cursors.iter_mut() {
                // Update cursor with offset
                c.range.0 += offset;
                c.range.1 += offset;
                
                // Figure out how many spaces to insert
                let (_, vis_pos) = self.buffer.index_to_v2d(c.range.0);
                let next_tab_stop = ((vis_pos / self.buffer.tab_width) + 1) * self.buffer.tab_width;
                let space_count = min(next_tab_stop - vis_pos, 8);
                
                
                // Insert spaces
                let space_strs = ["", " ", "  ", "   ", "    ", "     ", "      ", "       ", "        "];
                self.buffer.insert_text(space_strs[space_count], c.range.0);
                self.dirty = true;
                
                // Move cursor
                c.range.0 += space_count;
                c.range.1 += space_count;
                c.update_vis_start(&(self.buffer));
                    
                // Update offset
                offset += space_count;
            }
            
            // Adjust view
            self.move_view_to_cursor();
        }
        else {
            self.insert_text_at_cursor("\t");
        }
    }
    
    pub fn backspace_at_cursor(&mut self) {
        self.remove_text_behind_cursor(1);
    }
    
    pub fn insert_text_at_grapheme(&mut self, text: &str, pos: usize) {
        self.dirty = true;
        let buf_len = self.buffer.grapheme_count();
        self.buffer.insert_text(text, if pos < buf_len {pos} else {buf_len});
    }
    
    pub fn remove_text_behind_cursor(&mut self, grapheme_count: usize) {
        self.cursors.make_consistent();
        
        let mut offset = 0;
        
        for c in self.cursors.iter_mut() {
            // Update cursor with offset
            c.range.0 -= offset;
            c.range.1 -= offset;
            
            // Do nothing if there's nothing to delete.
            if c.range.0 == 0 {
                continue;
            }
            
            let len = min(c.range.0, grapheme_count);
            
            // Remove text
            self.buffer.remove_text_before(c.range.0, len);
            self.dirty = true;
            
            // Move cursor
            c.range.0 -= len;
            c.range.1 -= len;
            c.update_vis_start(&(self.buffer));
            
            // Update offset
            offset += len;
        }
        
        self.cursors.make_consistent();
        
        // Adjust view
        self.move_view_to_cursor();
    }
    
    pub fn remove_text_in_front_of_cursor(&mut self, grapheme_count: usize) {
        self.cursors.make_consistent();
        
        let mut offset = 0;
        
        for c in self.cursors.iter_mut() {
            // Update cursor with offset
            c.range.0 -= min(c.range.0, offset);
            c.range.1 -= min(c.range.1, offset);
        
            // Do nothing if there's nothing to delete.
            if c.range.1 == self.buffer.grapheme_count() {
                return;
            }
            
            let max_len = if self.buffer.grapheme_count() > c.range.1 {self.buffer.grapheme_count() - c.range.1} else {0};
            let len = min(max_len, grapheme_count);
            
            // Remove text
            self.buffer.remove_text_after(c.range.1, len);
            self.dirty = true;
            
            // Move cursor
            c.update_vis_start(&(self.buffer));
            
            // Update offset
            offset += len;
        }
        
        self.cursors.make_consistent();
        
        // Adjust view
        self.move_view_to_cursor();
    }
    
    pub fn remove_text_inside_cursor(&mut self) {
        self.cursors.make_consistent();
        
        let mut offset = 0;
        
        for c in self.cursors.iter_mut() {
            // Update cursor with offset
            c.range.0 -= min(c.range.0, offset);
            c.range.1 -= min(c.range.1, offset);
        
            // If selection, remove text
            if c.range.0 < c.range.1 {
                let len = c.range.1 - c.range.0;
                
                self.buffer.remove_text_before(c.range.0, c.range.1 - c.range.0);
                self.dirty = true;
            
                // Move cursor
                c.range.1 = c.range.0;
                
                // Update offset
                offset += len;
            }
            
            c.update_vis_start(&(self.buffer));
        }
        
        self.cursors.make_consistent();
        
        // Adjust view
        self.move_view_to_cursor();
    }
    
    pub fn cursor_to_beginning_of_buffer(&mut self) {
        self.cursors = CursorSet::new();
        
        self.cursors[0].range = (0, 0);
        self.cursors[0].update_vis_start(&(self.buffer));
        
        // Adjust view
        self.move_view_to_cursor();
    }
    
    pub fn cursor_to_end_of_buffer(&mut self) {
        let end = self.buffer.grapheme_count();
        
        self.cursors = CursorSet::new();
        self.cursors[0].range = (end, end);
        self.cursors[0].update_vis_start(&(self.buffer));
        
        // Adjust view
        self.move_view_to_cursor();
    }
    
    pub fn cursor_left(&mut self, n: usize) {
        for c in self.cursors.iter_mut() {
            if c.range.0 >= n {
                c.range.0 -= n;
            }
            else {
                c.range.0 = 0;
            }
            
            c.range.1 = c.range.0;
            c.update_vis_start(&(self.buffer));
        }
        
        // Adjust view
        self.move_view_to_cursor();
    }
    
    pub fn cursor_right(&mut self, n: usize) {
        for c in self.cursors.iter_mut() {
            c.range.1 += n;
            
            if c.range.1 > self.buffer.grapheme_count() {
                c.range.1 = self.buffer.grapheme_count();
            }
            
            c.range.0 = c.range.1;
            c.update_vis_start(&(self.buffer));
        }
        
        // Adjust view
        self.move_view_to_cursor();
    }
    
    pub fn cursor_up(&mut self, n: usize) {
        for c in self.cursors.iter_mut() {
            let (v, _) = self.buffer.index_to_v2d(c.range.0);
            
            if v >= n {
                c.range.0 = self.buffer.v2d_to_index((v - n, c.vis_start));
                c.range.1 = c.range.0;
            }
            else {
                c.range = (0, 0);
                c.update_vis_start(&(self.buffer));
            }
        }
        
        // Adjust view
        self.move_view_to_cursor();
    }
    
    pub fn cursor_down(&mut self, n: usize) {
        for c in self.cursors.iter_mut() {
            let (v, _) = self.buffer.index_to_v2d(c.range.0);
            
            if v < (self.buffer.line_count() - n) {
                c.range.0 = self.buffer.v2d_to_index((v + n, c.vis_start));
                c.range.1 = c.range.0;
            }
            else {
                let end = self.buffer.grapheme_count();
                c.range = (end, end);
                c.update_vis_start(&(self.buffer));
            }
        }
        
        // Adjust view
        self.move_view_to_cursor();
    }
    
    pub fn page_up(&mut self) {
        if self.view_pos.0 > 0 {
            let move_amount = self.view_dim.0 - (self.view_dim.0 / 8);
            if self.view_pos.0 >= move_amount {
                self.view_pos.0 -= move_amount;
            }
            else {
                self.view_pos.0 = 0;
            }
            
            self.cursor_up(move_amount);
        }
        else {
            self.cursor_to_beginning_of_buffer();
        }
        
        // Adjust view
        self.move_view_to_cursor();
    }
    
    pub fn page_down(&mut self) {
        let nlc = self.buffer.line_count() - 1;
        
        if self.view_pos.0 < nlc {
            let move_amount = self.view_dim.0 - (self.view_dim.0 / 8);
            let max_move = nlc - self.view_pos.0;
            
            if max_move >= move_amount {
                self.view_pos.0 += move_amount;
            }
            else {
                self.view_pos.0 += max_move;
            }
            
            self.cursor_down(move_amount);
        }
        else {
            self.cursor_to_end_of_buffer();
        }
        
        // Adjust view
        self.move_view_to_cursor();
    }
    
    pub fn jump_to_line(&mut self, n: usize) {
        let pos = self.buffer.line_col_to_index((n, 0));
        let (v, _) = self.buffer.index_to_v2d(pos);
        self.cursors.truncate(1);
        self.cursors[0].range.0 = self.buffer.v2d_to_index((v, self.cursors[0].vis_start));
        self.cursors[0].range.1 = self.cursors[0].range.0;
        
        // Adjust view
        self.move_view_to_cursor();
    }
}