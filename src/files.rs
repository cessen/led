use std::old_io::{IoResult, BufferedReader, BufferedWriter};
use std::old_io::fs::File;
use std::path::Path;

use buffer::line::{Line, LineEnding, line_ending_to_str};
use buffer::line_formatter::LineFormatter;
use buffer::Buffer as TextBuffer;

pub fn load_file_to_buffer<T: LineFormatter>(path: &Path, lf: T) -> IoResult<TextBuffer<T>> {
    let mut tb = TextBuffer::new(lf);
    let mut f = BufferedReader::new(try!(File::open(path)));
    
    for line in f.lines() {
        let l = Line::new_from_string_unchecked(line.unwrap());
        if l.ending != LineEnding::None {
            tb.line_ending_type = l.ending;
        }
        tb.append_line_unchecked(l);
    }
    
    // Remove initial blank line
    tb.remove_lines(0, 1);
    
    return Ok(tb);
}

pub fn save_buffer_to_file<T: LineFormatter>(tb: &TextBuffer<T>, path: &Path) -> IoResult<()> {
    // TODO: make save atomic
    let mut iter = tb.line_iter();
    let mut f = BufferedWriter::new(try!(File::create(path)));
    
    for l in iter {
        let _ = f.write_str(l.as_str());
        let _ = f.write_str(line_ending_to_str(l.ending));
    }
    
    return Ok(());
}