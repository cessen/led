extern crate rustbox;
extern crate docopt;
extern crate serialize;

use std::char;
use std::path::Path;
use docopt::Docopt;
use editor::Editor;
use term_ui::draw_editor;

mod buffer;
mod files;
mod editor;
mod term_ui;


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
//const K_BACKSPACE: u16 = 127;
const K_DOWN: u16 = 65516;
const K_LEFT: u16 = 65515;
const K_RIGHT: u16 = 65514;
const K_UP: u16 = 65517;
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
    let mut editor = if let Option::Some(s) = args.arg_file {
        Editor::new_from_file(&Path::new(s.as_slice()))
    }
    else {
        Editor::new()
    };
    
    rustbox::init();
    
    loop {
        // Draw the editor to screen
        rustbox::clear();
        draw_editor(&editor, (0, 0), (height-1, width-1));
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
                            editor.save_if_dirty();
                        },
                        
                        K_UP => {
                            editor.cursor_up();
                        },
                        
                        K_DOWN => {
                            editor.cursor_down();
                        },
                        
                        K_LEFT => {
                            editor.cursor_left();
                        },
                        
                        K_RIGHT => {
                            editor.cursor_right();
                        },
                        
                        K_ENTER => {
                            editor.insert_text_at_cursor("\n");
                        },
                        
                        K_SPACE => {
                            editor.insert_text_at_cursor(" ");
                        },
                        
                        K_TAB => {
                            editor.insert_text_at_cursor("\t");
                        },
                        
                        // Character
                        0 => {
                            if let Option::Some(c) = char::from_u32(character) {
                                editor.insert_text_at_cursor(c.to_string().as_slice());
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
    
    //println!("{}", editor.buffer.root.tree_height);
}
