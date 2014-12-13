use std::cmp::min;


/// A block of text, contiguous in memory
pub struct TextBlock {
    pub data: Vec<u8>,
}

impl TextBlock {
    pub fn new() -> TextBlock {
        TextBlock {
            data: Vec::<u8>::new()
        }
    }
    
    pub fn insert_text(&mut self, text: &str, pos: uint) {
        let ins = min(pos, self.data.len());

        // Grow data size		
        self.data.grow(text.len(), 0);
        
        // Move old bytes forward
        let mut from = self.data.len() - text.len();
        let mut to = self.data.len();
        while from > ins {
            from -= 1;
            to -= 1;
            
            self.data[to] = self.data[from];
        }
        
        // Copy new bytes in
        let mut i = ins;
        for b in text.bytes() {
            self.data[i] = b;
            i += 1
        }
    }
}


// /// A rope text storage buffer, using TextBlocks for its underlying text
// /// storage.
// pub enum TextBuffer {
//     Block(TextBlock),
//     Node(Box<TextBuffer>, Box<TextBuffer>)
// }
// 
// impl TextBuffer {
//     pub fn new() -> TextBuffer {
//         TextBuffer::Block(TextBlock::new())
//     }
// }