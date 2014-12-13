extern crate rustbox;
extern crate docopt;
extern crate serialize;

use docopt::Docopt;
use buffer::TextBuffer; 

mod buffer;


// Usage documentation string
static USAGE: &'static str = "
Usage: led <file>
       led --help

Options:
    -h, --help  Show this message
";


// Struct for storing command-line arguments
#[deriving(Decodable, Show)]
    struct Args {
    arg_file: String,
    flag_help: bool,
}


fn main() {
    // Get command-line arguments
    let args: Args = Docopt::new(USAGE).and_then(|d| d.decode()).unwrap_or_else(|e| e.exit());
    
    let mut tb = TextBuffer::new();

    for _ in range(0i, 100) {
        tb.insert_text(args.arg_file.as_slice(), 0);
        //if tb.len() > 1024 {
        //    tb.remove_text(0, args.arg_file.len());
        //}
    }
    
    println!("{}", tb);
}