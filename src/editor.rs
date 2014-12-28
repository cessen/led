#![allow(dead_code)]

use buffer::TextBuffer;
use std::path::Path;
use files::{load_file_to_buffer, save_buffer_to_file};
use string_utils::char_count;


pub struct Editor {
    pub buffer: TextBuffer,
    pub file_path: Path,
    pub dirty: bool,
    
    // The dimensions and position of the editor's view within the buffer
    pub view_dim: (uint, uint),  // (height, width)
    pub view_pos: (uint, uint),  // (line, col)
    
    // The editing cursor position
    pub cursor: (uint, uint),  // (line, col)
}


impl Editor {
    /// Create a new blank editor
    pub fn new() -> Editor {
        Editor {
            buffer: TextBuffer::new(),
            file_path: Path::new(""),
            dirty: false,
            view_dim: (0, 0),
            view_pos: (0, 0),
            cursor: (0, 0),
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
            cursor: (0, 0),
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
        // Horizontal
        if self.cursor.1 < self.view_pos.1 {
            self.view_pos.1 = self.cursor.1;
        }
        else if self.cursor.1 >= (self.view_pos.1 + self.view_dim.1) {
            self.view_pos.1 = 1 + self.cursor.1 - self.view_dim.1;
        }
        
        // Vertical
        if self.cursor.0 < self.view_pos.0 {
            self.view_pos.0 = self.cursor.0;
        }
        else if self.cursor.0 >= (self.view_pos.0 + self.view_dim.0) {
            self.view_pos.0 = 1 + self.cursor.0 - self.view_dim.0;
        }
    }
    
    pub fn insert_text_at_cursor(&mut self, text: &str) {
        let pos = self.buffer.pos_2d_to_closest_1d(self.cursor);
        let str_len = char_count(text);
        let p = self.buffer.pos_2d_to_closest_1d(self.cursor);
        
        // Insert text
        self.buffer.insert_text(text, pos);
        self.dirty = true;
        
        // Move cursor
        self.cursor = self.buffer.pos_1d_to_closest_2d(p + str_len);
        
        self.move_view_to_cursor();
    }
    
    pub fn insert_text_at_char(&mut self, text: &str, pos: uint) {
        self.dirty = true;
        let buf_len = self.buffer.len();
        self.buffer.insert_text(text, if pos < buf_len {pos} else {buf_len});
    }
    
    pub fn remove_text_behind_cursor(&mut self, char_count: uint) {
        let pos_b = self.buffer.pos_2d_to_closest_1d(self.cursor);
        let pos_a = if pos_b >= char_count {pos_b - char_count} else {0};
        
        // Move cursor
        self.cursor = self.buffer.pos_1d_to_closest_2d(pos_a);
        
        // Remove text
        self.buffer.remove_text(pos_a, pos_b);
        
        self.dirty = true;
        
        self.move_view_to_cursor();
    }
    
    pub fn cursor_to_beginning_of_buffer(&mut self) {
        self.cursor = (0, 0);
    }
    
    pub fn cursor_to_end_of_buffer(&mut self) {
        self.cursor = self.buffer.pos_1d_to_closest_2d(self.buffer.len()+1);
    }
    
    pub fn cursor_left(&mut self) {
        let p = self.buffer.pos_2d_to_closest_1d(self.cursor);

        if p > 0 {
            self.cursor = self.buffer.pos_1d_to_closest_2d(p - 1);
        }
        else {
            self.cursor = self.buffer.pos_1d_to_closest_2d(0);
        }
        
        self.move_view_to_cursor();
    }
    
    pub fn cursor_right(&mut self) {
        let p = self.buffer.pos_2d_to_closest_1d(self.cursor);
        self.cursor = self.buffer.pos_1d_to_closest_2d(p + 1);
        
        self.move_view_to_cursor();
    }
    
    pub fn cursor_up(&mut self) {
        if self.cursor.0 > 0 {
            self.cursor.0 -= 1;
        }
        else {
            self.cursor_to_beginning_of_buffer();
        }
        
        self.move_view_to_cursor();
    }
    
    pub fn cursor_down(&mut self) {
        if self.cursor.0 < self.buffer.newline_count() {
            self.cursor.0 += 1;
        }
        else {
            self.cursor_to_end_of_buffer();
        }
        
        self.move_view_to_cursor();
    }
    
    pub fn page_up(&mut self) {
        if self.view_pos.0 > 0 {
            let move_amount = self.view_dim.0 - (self.view_dim.0 / 8);
            if self.view_pos.0 >= move_amount {
                if self.cursor.0 >= move_amount {
                    self.cursor.0 -= move_amount;
                }
                self.view_pos.0 -= move_amount;
            }
            else {
                if self.cursor.0 >= self.view_pos.0 {
                    self.cursor.0 -= self.view_pos.0;
                }
                else {
                    self.cursor_to_beginning_of_buffer();
                }
                self.view_pos.0 = 0;
            }   
        }
        else {
            self.cursor_to_beginning_of_buffer();
        }
        
        self.move_view_to_cursor();
    }
    
    pub fn page_down(&mut self) {
        let nlc = self.buffer.newline_count();
        
        if self.view_pos.0 < nlc {
            let move_amount = self.view_dim.0 - (self.view_dim.0 / 8);
            let max_move = nlc - self.view_pos.0;
            let cursor_max_move = nlc - self.cursor.0;
            
            if max_move >= move_amount {
                self.view_pos.0 += move_amount;
            }
            else {
                self.view_pos.0 += max_move;
            }
            
            if cursor_max_move >= move_amount {
                self.cursor.0 += move_amount;
            }
            else {
                self.cursor_to_end_of_buffer();
            }
        }
        
        self.move_view_to_cursor();
    }
}
