extern crate rustbox;
extern crate docopt;
extern crate serialize;


use std::char;
use std::path::Path;
use docopt::Docopt;
use buffer::TextBuffer; 
use rustbox::{Style,Color};
use files::{load_file_to_buffer, save_buffer_to_file};

mod buffer;
mod files;


// Usage documentation string
static USAGE: &'static str = "
Usage: led [<file>]
       led --help

Options:
    -h, --help  Show this message
";


// Struct for storing command-line arguments
#[deriving(Decodable, Show)]
    struct Args {
    arg_file: Option<String>,
    flag_help: bool,
}


// Key codes
const K_ENTER: u16 = 13;
const K_TAB: u16 = 9;
const K_SPACE: u16 = 32;
const K_BACKSPACE: u16 = 127;
//const K_DOWN: u16 = 65516;
//const K_LEFT: u16 = 65515;
//const K_RIGHT: u16 = 65514;
//const K_UP: u16 = 65517;
const K_ESC: u16 = 27;
const K_CTRL_Q: u16 = 17;
const K_CTRL_S: u16 = 19;



fn main() {
    // Get command-line arguments
    let args: Args = Docopt::new(USAGE).and_then(|d| d.decode()).unwrap_or_else(|e| e.exit());
    
    // Quitting flag
    let mut quit = false;
    
    let mut width = rustbox::width();
    let mut height = rustbox::height();

    // Load file, if specified    
    let mut tb = if let Option::Some(s) = args.arg_file {
        load_file_to_buffer(&Path::new(s.as_slice())).unwrap()
    }
    else {
        TextBuffer::new()
    };
    
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
                    rustbox::print(column, line, Style::Normal, Color::White, Color::Black, c.to_string());
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
        
        
        // Handle events.  We block on the first event, so that the
        // program doesn't loop like crazy, but then continue pulling
        // events in a non-blocking way until we run out of events
        // to handle.
        let mut e = rustbox::poll_event(); // Block until we get an event
        loop {
            match e {
                rustbox::Event::KeyEvent(modifier, key, character) => {
                    //println!("      {} {} {}", modifier, key, character);
                    match key {
                        K_CTRL_Q | K_ESC => {
                            quit = true;
                            break;
                        },
                        
                        K_CTRL_S => {
                            save_buffer_to_file(&tb, &Path::new("untitled.txt"));
                        },
                        
                        K_ENTER => {
                            let p = tb.len();
                            tb.insert_text("\n", p);
                        },
                        
                        K_SPACE => {
                            let p = tb.len();
                            tb.insert_text(" ", p);
                        },
                        
                        K_TAB => {
                            let p = tb.len();
                            tb.insert_text("\t", p);
                        },
                        
                        // Character
                        0 => {
                            if let Option::Some(c) = char::from_u32(character) {
                                let p = tb.len();
                                tb.insert_text(c.to_string().as_slice(), p);
                            }
                        },
                        
                        _ => {}
                    }
                },
                
                rustbox::Event::ResizeEvent(w, h) => {
                    width = w as uint;
                    height = h as uint;
                }
                
                _ => {
                    break;
                }
            }

            e = rustbox::peek_event(0); // Get next event (if any)
        }
        
        
        // Quit if quit flag is set
        if quit {
            break;
        }
    }
    
    rustbox::shutdown();
    
    println!("{}", tb.root.tree_height);
}
