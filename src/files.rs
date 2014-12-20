use std::io::{IoResult, BufferedReader, BufferedWriter};
use std::io::fs::File;
use std::path::Path;

use buffer::TextBuffer;

pub fn load_file_to_buffer(path: &Path) -> IoResult<TextBuffer> {
    let mut tb = TextBuffer::new();
    let mut f = BufferedReader::new(try!(File::open(path)));
    
    loop {
        let line = f.read_line();
        if let Ok(ref s) = line {
            let tbl = tb.len(); 
            tb.insert_text(s.as_slice(), tbl);
        }
        else {
            break;
        }
    }
    
    return Ok(tb);
}

pub fn save_buffer_to_file(tb: &TextBuffer, path: &Path) -> IoResult<()> {
    let mut iter = tb.root_iter();
    let mut f = BufferedWriter::new(try!(File::create(path)));
    
    for c in iter {
        f.write_char(c);
    }
    
    return Ok(());
}