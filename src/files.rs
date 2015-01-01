use std::io::{IoResult, BufferedReader, BufferedWriter};
use std::io::fs::File;
use std::path::Path;

use buffer::line::{Line, LineEnding, line_ending_to_str};
use buffer::Buffer as TextBuffer;

pub fn load_file_to_buffer(path: &Path) -> IoResult<TextBuffer> {
    let mut tb = TextBuffer::new();
    let mut f = BufferedReader::new(try!(File::open(path)));
    let mut last_line_breaks = true;
    
    for line in f.lines() {
        let l = Line::new_from_string_unchecked(line.unwrap());
        last_line_breaks = l.ending != LineEnding::None;
        tb.append_line_unchecked(l);
    }
    
    // If the last line had a line break, we need to add a final
    // blank line.
    if last_line_breaks {
        tb.append_line_unchecked(Line::new());
    }
    
    // Remove initial blank line
    tb.remove_lines(0, 1);
    
    return Ok(tb);
}

pub fn save_buffer_to_file(tb: &TextBuffer, path: &Path) -> IoResult<()> {
    // TODO: make save atomic
    let mut iter = tb.line_iter();
    let mut f = BufferedWriter::new(try!(File::create(path)));
    
    for l in iter {
        let _ = f.write_str(l.as_str());
        let _ = f.write_str(line_ending_to_str(l.ending));
    }
    
    return Ok(());
}