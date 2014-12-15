extern crate rustbox;
extern crate docopt;
extern crate serialize;

use std::char;
use docopt::Docopt;
use buffer::TextBuffer; 
use rustbox::{Style,Color};

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
    
    let mut width = rustbox::width();
    let mut height = rustbox::height();
    
    let mut tb = TextBuffer::new();
    
    rustbox::init();

    loop {
        // Draw the text buffer to screen
        rustbox::clear();
        {
            let mut tb_iter = tb.root_iter();
            let mut line: uint = 0;
            let mut column: uint = 0;
            
            loop {
                if let Option::Some(c) = tb_iter.next() {
                    if c == '\n' {
                        line += 1;
                        column = 0;
                        continue;
                    }
                    rustbox::print(column, line, Style::Bold, Color::White, Color::Black, c.to_string());
                    column += 1;
                }
                else {
                    break;
                }
                
                if line > height {
                    break;
                }
                
                if column > width {
                    tb_iter.next_line();
                    line += 1;
                    column = 0;
                }
            }
        }
        rustbox::present();
        
        // Handle events
        match rustbox::poll_event() {
            rustbox::Event::KeyEvent(_, _, ch) => {
                match char::from_u32(ch) {
                    Some('q') => { break; },
                    
                    Some('w') => {
                        let p = tb.len();
                        tb.insert_text("\n", p);
                    },
                    
                    Some(c) => {
                        let p = tb.len();
                        tb.insert_text(c.to_string().as_slice(), p);
                    },
                    
                    _ => {}
                }
            },
            
            rustbox::Event::ResizeEvent(w, h) => {
                width = w as uint;
                height = h as uint;
            }
            
            _ => { }
        }
    }
    rustbox::shutdown();
}
