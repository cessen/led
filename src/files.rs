use std::old_io::{IoResult, BufferedWriter};
use std::old_io::fs::File;
use std::path::Path;

use buffer::line::{line_ending_to_str};
use buffer::line_formatter::LineFormatter;
use buffer::Buffer as TextBuffer;

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
