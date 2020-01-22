use ropey::Rope;

/// An open text buffer, currently being edited.
#[derive(Debug, Clone)]
pub struct Buffer {
    pub is_dirty: bool, // Is this buffer currently out of sync with disk.
    pub text: Rope,     // The actual text content.
}

impl Buffer {
    pub fn new(text: Rope) -> Buffer {
        Buffer {
            is_dirty: false,
            text: text,
        }
    }

    pub fn insert(&mut self, text: &str, char_idx: usize) {
        self.text.insert(char_idx, text);
        self.is_dirty = true;
    }

    pub fn remove(&mut self, char_start: usize, char_end: usize) {
        self.text.remove(char_start..char_end);
        self.is_dirty = true;
    }
}
