use std::old_io::{IoResult, BufferedWriter};
use std::old_io::fs::File;
use std::old_path::Path;

use buffer::line::{line_ending_to_str};
use buffer::Buffer as TextBuffer;


pub fn save_buffer_to_file(tb: &TextBuffer, path: &Path) -> IoResult<()> {
    // TODO: make save atomic
    let mut f = BufferedWriter::new(try!(File::create(path)));
    
    for l in tb.line_iter() {
        let _ = f.write_str(l.as_str());
        let _ = f.write_str(line_ending_to_str(l.ending));
    }
    
    return Ok(());
}
