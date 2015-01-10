#![feature(old_orphan_check)]  // Temporary, get rid of this once the new orphan check works well
#![allow(unstable)]

extern crate rustbox;
extern crate docopt;
extern crate "rustc-serialize" as rustc_serialize;

use std::path::Path;
use docopt::Docopt;
use editor::Editor;
use term_ui::TermUI;

mod string_utils;
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
#[derive(RustcDecodable, Show)]
    struct Args {
    arg_file: Option<String>,
    flag_help: bool,
}




fn main() {
    // Get command-line arguments
    let args: Args = Docopt::new(USAGE).and_then(|d| d.decode()).unwrap_or_else(|e| e.exit());
    
    // Load file, if specified    
    let editor = if let Option::Some(s) = args.arg_file {
        Editor::new_from_file(&Path::new(s.as_slice()))
    }
    else {
        Editor::new()
    };
        
    // Initialize and start UI
    let mut ui = TermUI::new_from_editor(editor);
    ui.main_ui_loop();
    
    //println!("{}", editor.buffer.root.tree_height);
}
