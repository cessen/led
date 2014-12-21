#![allow(dead_code)]

use buffer::TextBuffer;
use std::path::Path;
use files::{load_file_to_buffer, save_buffer_to_file};


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
        let mut buf = load_file_to_buffer(path).unwrap();
        
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
    
    pub fn insert_text_at_cursor(&mut self, text: &str) {
        let pos = self.buffer.pos_2d_to_closest_1d(self.cursor);
        
        self.buffer.insert_text(text, pos);
        
        self.dirty = true;
        
        if text == "\n" {
            self.cursor.0 += 1;
            self.cursor.1 = 0;
        }
        else {
            self.cursor.1 += 1;
        }
    }
    
    pub fn cursor_left(&mut self) {
        if self.cursor.1 > 0 {
            self.cursor.1 -= 1;
        }
    }
    
    pub fn cursor_right(&mut self) {
        self.cursor.1 += 1;
    }
    
    pub fn cursor_up(&mut self) {
        if self.cursor.0 > 0 {
            self.cursor.0 -= 1;
        }
    }
    
    pub fn cursor_down(&mut self) {
        self.cursor.0 += 1;
    }
}
