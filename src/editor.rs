#![allow(dead_code)]

use buffer::Buffer;
use std::path::Path;
use files::{load_file_to_buffer, save_buffer_to_file};
use string_utils::grapheme_count;


/// A text cursor.  Also represents selections when range.0 != range.1.
///
/// `range` is a pair of 1d grapheme indexes into the text.
///
/// `vis_start` is the visual 2d horizontal position of the cursor.  This
/// doesn't affect editing operations at all, but is used for cursor movement.
pub struct Cursor {
    pub range: (uint, uint),  // start, end
    pub vis_start: uint,  // start
}

impl Cursor {
    pub fn new() -> Cursor {
        Cursor {
            range: (0, 0),
            vis_start: 0,
        }
    }
    
    pub fn update_vis_start(&mut self, buf: &Buffer) {
        let (v, h) = buf.pos_1d_to_closest_2d(self.range.0);
        self.vis_start = buf.get_line(v).grapheme_index_to_closest_vis_pos(h);
    }
}


pub struct Editor {
    pub buffer: Buffer,
    pub file_path: Path,
    pub dirty: bool,
    
    // The dimensions and position of the editor's view within the buffer
    pub view_dim: (uint, uint),  // (height, width)
    pub view_pos: (uint, uint),  // (line, col)
    
    // The editing cursor position
    pub cursor: Cursor,  
}


impl Editor {
    /// Create a new blank editor
    pub fn new() -> Editor {
        Editor {
            buffer: Buffer::new(),
            file_path: Path::new(""),
            dirty: false,
            view_dim: (0, 0),
            view_pos: (0, 0),
            cursor: Cursor::new(),
        }
    }
    
    pub fn new_from_file(path: &Path) -> Editor {
        let buf = load_file_to_buffer(path).unwrap();
        
        Editor {
            buffer: buf,
            file_path: path.clone(),
            dirty: false,
            view_dim: (0, 0),
            view_pos: (0, 0),
            cursor: Cursor::new(),
        }
    }
    
    pub fn save_if_dirty(&mut self) {
        if self.dirty && self.file_path != Path::new("") {
            let _ = save_buffer_to_file(&self.buffer, &self.file_path);
            self.dirty = false;
        }
    }
    
    pub fn update_dim(&mut self, h: uint, w: uint) {
        self.view_dim = (h, w);
    }
    
    
    /// Moves the editor's view the minimum amount to show the cursor
    pub fn move_view_to_cursor(&mut self) {
        let (v, h) = self.buffer.pos_1d_to_closest_vis_2d(self.cursor.range.0);
        
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
    
    pub fn insert_text_at_grapheme(&mut self, text: &str, pos: uint) {
        self.dirty = true;
        let buf_len = self.buffer.len();
        self.buffer.insert_text(text, if pos < buf_len {pos} else {buf_len});
    }
    
    pub fn remove_text_behind_cursor(&mut self, grapheme_count: uint) {
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
    
    pub fn remove_text_in_front_of_cursor(&mut self, grapheme_count: uint) {
        let pos_a = self.cursor.range.1;
        let pos_b = if (pos_a + grapheme_count) <= self.buffer.len() {pos_a + grapheme_count} else {self.buffer.len()};
        
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
        let end = self.buffer.len();
        self.cursor.range = (end, end);
        self.cursor.update_vis_start(&(self.buffer));
        
        // Adjust view
        self.move_view_to_cursor();
    }
    
    pub fn cursor_left(&mut self, n: uint) {
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
    
    pub fn cursor_right(&mut self, n: uint) {
        if self.cursor.range.1 <= (self.buffer.len() - n) {
            self.cursor.range.1 += n;
        }
        else {
            self.cursor.range.1 = self.buffer.len();
        }
        
        self.cursor.range.0 = self.cursor.range.1;
        self.cursor.update_vis_start(&(self.buffer));
        
        // Adjust view
        self.move_view_to_cursor();
    }
    
    pub fn cursor_up(&mut self, n: uint) {
        let (v, _) = self.buffer.pos_1d_to_closest_vis_2d(self.cursor.range.0);
        
        if v >= n {
            self.cursor.range.0 = self.buffer.pos_vis_2d_to_closest_1d((v - n, self.cursor.vis_start));
            self.cursor.range.1 = self.cursor.range.0;
        }
        else {
            self.cursor_to_beginning_of_buffer();
        }
        
        // Adjust view
        self.move_view_to_cursor();
    }
    
    pub fn cursor_down(&mut self, n: uint) {
        let (v, _) = self.buffer.pos_1d_to_closest_vis_2d(self.cursor.range.0);
        
        if v < (self.buffer.line_count() - n) {
            self.cursor.range.0 = self.buffer.pos_vis_2d_to_closest_1d((v + n, self.cursor.vis_start));
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
    
    pub fn jump_to_line(&mut self, n: uint) {
        let pos = self.buffer.pos_2d_to_closest_1d((n, 0));
        let (v, _) = self.buffer.pos_1d_to_closest_vis_2d(pos);
        self.cursor.range.0 = self.buffer.pos_vis_2d_to_closest_1d((v, self.cursor.vis_start));
        self.cursor.range.1 = self.cursor.range.0;
        
        // Adjust view
        self.move_view_to_cursor();
    }
}
