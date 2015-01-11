#![allow(dead_code)]

use buffer::Buffer;
use std::path::Path;
use std::cmp::min;
use files::{load_file_to_buffer, save_buffer_to_file};
use string_utils::grapheme_count;


/// A text cursor.  Also represents selections when range.0 != range.1.
///
/// `range` is a pair of 1d grapheme indexes into the text.
///
/// `vis_start` is the visual 2d horizontal position of the cursor.  This
/// doesn't affect editing operations at all, but is used for cursor movement.
pub struct Cursor {
    pub range: (usize, usize),  // start, end
    pub vis_start: usize,  // start
}

impl Cursor {
    pub fn new() -> Cursor {
        Cursor {
            range: (0, 0),
            vis_start: 0,
        }
    }
    
    pub fn update_vis_start(&mut self, buf: &Buffer) {
        let (_, h) = buf.index_to_v2d(self.range.0);
        self.vis_start = h;
    }
}


pub struct Editor {
    pub buffer: Buffer,
    pub file_path: Path,
    pub soft_tabs: bool,
    pub dirty: bool,
    
    // The dimensions and position of the editor's view within the buffer
    pub view_dim: (usize, usize),  // (height, width)
    pub view_pos: (usize, usize),  // (line, col)
    
    // The editing cursor position
    pub cursor: Cursor,  
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
            cursor: Cursor::new(),
        }
    }
    
    pub fn new_from_file(path: &Path) -> Editor {
        let buf = load_file_to_buffer(path).unwrap();
        
        let mut ed = Editor {
            buffer: buf,
            file_path: path.clone(),
            soft_tabs: false,
            dirty: false,
            view_dim: (0, 0),
            view_pos: (0, 0),
            cursor: Cursor::new(),
        };
        
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
        if let Some(pos) = self.buffer.undo() {
            self.cursor.range.0 = pos;
            self.cursor.range.1 = pos;
            self.cursor.update_vis_start(&(self.buffer));
            
            self.move_view_to_cursor();
        }
    }
    
    
    pub fn redo(&mut self) {
        if let Some(pos) = self.buffer.redo() {
            self.cursor.range.0 = pos;
            self.cursor.range.1 = pos;
            self.cursor.update_vis_start(&(self.buffer));
            
            self.move_view_to_cursor();
        }
    }
    
    
    /// Moves the editor's view the minimum amount to show the cursor
    pub fn move_view_to_cursor(&mut self) {
        let (v, h) = self.buffer.index_to_v2d(self.cursor.range.0);
        
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
        let str_len = grapheme_count(text);
        
        // Insert text
        self.buffer.insert_text(text, self.cursor.range.0);
        self.dirty = true;
        
        // Move cursor
        self.cursor.range.0 += str_len;
        self.cursor.range.1 += str_len;
        self.cursor.update_vis_start(&(self.buffer));
        
        // Adjust view
        self.move_view_to_cursor();
    }
    
    pub fn insert_tab_at_cursor(&mut self) {
        if self.soft_tabs {
            // Figure out how many spaces to insert
            let (_, vis_pos) = self.buffer.index_to_v2d(self.cursor.range.0);
            let next_tab_stop = ((vis_pos / self.buffer.tab_width) + 1) * self.buffer.tab_width;
            let space_count = min(next_tab_stop - vis_pos, 8);
            
            
            // Insert spaces
            let space_strs = ["", " ", "  ", "   ", "    ", "     ", "      ", "       ", "        "];
            self.buffer.insert_text(space_strs[space_count], self.cursor.range.0);
            self.dirty = true;
            
            // Move cursor
            self.cursor.range.0 += space_count;
            self.cursor.range.1 += space_count;
            self.cursor.update_vis_start(&(self.buffer));
            
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
        // Do nothing if there's nothing to delete.
        if self.cursor.range.0 == 0 {
            return;
        }
        
        let pos_b = self.cursor.range.0;
        let pos_a = if pos_b >= grapheme_count {pos_b - grapheme_count} else {0};
        let tot_g = pos_b - pos_a;
        
        // Remove text
        self.buffer.remove_text(pos_a, pos_b);
        self.dirty = true;
        
        // Move cursor
        self.cursor.range.0 -= tot_g;
        self.cursor.range.1 -= tot_g;
        self.cursor.update_vis_start(&(self.buffer));
        
        // Adjust view
        self.move_view_to_cursor();
    }
    
    pub fn remove_text_in_front_of_cursor(&mut self, grapheme_count: usize) {
        // Do nothing if there's nothing to delete.
        if self.cursor.range.0 == self.buffer.grapheme_count() {
            return;
        }
        
        let pos_a = self.cursor.range.1;
        let pos_b = if (pos_a + grapheme_count) <= self.buffer.grapheme_count() {pos_a + grapheme_count} else {self.buffer.grapheme_count()};
        
        // Remove text
        self.buffer.remove_text(pos_a, pos_b);
        self.dirty = true;
        
        // Move cursor
        self.cursor.update_vis_start(&(self.buffer));
        
        // Adjust view
        self.move_view_to_cursor();
    }
    
    pub fn remove_text_inside_cursor(&mut self) {
        // If selection, remove text
        if self.cursor.range.0 < self.cursor.range.1 {
            self.buffer.remove_text(self.cursor.range.0, self.cursor.range.1);
            self.dirty = true;
        }
        
        // Move cursor
        self.cursor.range.1 = self.cursor.range.0;
        self.cursor.update_vis_start(&(self.buffer));
        
        // Adjust view
        self.move_view_to_cursor();
    }
    
    pub fn cursor_to_beginning_of_buffer(&mut self) {
        self.cursor.range = (0, 0);
        self.cursor.update_vis_start(&(self.buffer));
        
        // Adjust view
        self.move_view_to_cursor();
    }
    
    pub fn cursor_to_end_of_buffer(&mut self) {
        let end = self.buffer.grapheme_count();
        self.cursor.range = (end, end);
        self.cursor.update_vis_start(&(self.buffer));
        
        // Adjust view
        self.move_view_to_cursor();
    }
    
    pub fn cursor_left(&mut self, n: usize) {
        if self.cursor.range.0 >= n {
            self.cursor.range.0 -= n;
        }
        else {
            self.cursor.range.0 = 0;
        }
        
        self.cursor.range.1 = self.cursor.range.0;
        self.cursor.update_vis_start(&(self.buffer));
        
        // Adjust view
        self.move_view_to_cursor();
    }
    
    pub fn cursor_right(&mut self, n: usize) {
        if self.cursor.range.1 <= (self.buffer.grapheme_count() - n) {
            self.cursor.range.1 += n;
        }
        else {
            self.cursor.range.1 = self.buffer.grapheme_count();
        }
        
        self.cursor.range.0 = self.cursor.range.1;
        self.cursor.update_vis_start(&(self.buffer));
        
        // Adjust view
        self.move_view_to_cursor();
    }
    
    pub fn cursor_up(&mut self, n: usize) {
        let (v, _) = self.buffer.index_to_v2d(self.cursor.range.0);
        
        if v >= n {
            self.cursor.range.0 = self.buffer.v2d_to_index((v - n, self.cursor.vis_start));
            self.cursor.range.1 = self.cursor.range.0;
        }
        else {
            self.cursor_to_beginning_of_buffer();
        }
        
        // Adjust view
        self.move_view_to_cursor();
    }
    
    pub fn cursor_down(&mut self, n: usize) {
        let (v, _) = self.buffer.index_to_v2d(self.cursor.range.0);
        
        if v < (self.buffer.line_count() - n) {
            self.cursor.range.0 = self.buffer.v2d_to_index((v + n, self.cursor.vis_start));
            self.cursor.range.1 = self.cursor.range.0;
        }
        else {
            self.cursor_to_end_of_buffer();
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
        // TODO
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
        self.cursor.range.0 = self.buffer.v2d_to_index((v, self.cursor.vis_start));
        self.cursor.range.1 = self.cursor.range.0;
        
        // Adjust view
        self.move_view_to_cursor();
    }
}
