#![allow(unstable)]

extern crate rustbox;
extern crate docopt;
extern crate "rustc-serialize" as rustc_serialize;
extern crate freetype;
extern crate sdl2;

use std::path::Path;
use docopt::Docopt;
use editor::Editor;
use term_ui::TermUI;
//use gui::GUI;

mod string_utils;
mod buffer;
mod line_formatter;
mod files;
mod editor;
mod term_ui;
//mod font;
//mod gui;




// Usage documentation string
static USAGE: &'static str = "
Usage: led [options] [<file>]
       led --help

Options:
    -g, --gui   Use a graphical user interface instead of a console UI
    -h, --help  Show this message
";


// Struct for storing command-line arguments
#[derive(RustcDecodable, Show)]
    struct Args {
    arg_file: Option<String>,
    flag_gui: bool,
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
//    if args.flag_gui {
//        // GUI
//        sdl2::init(sdl2::INIT_VIDEO);
//        let mut ui = GUI::new_from_editor(editor);
//        ui.main_ui_loop();
//        sdl2::quit();
//    }
//    else {
        // Console UI
        let mut ui = TermUI::new_from_editor(editor);
        ui.main_ui_loop();
//    }
    
    //println!("{}", editor.buffer.root.tree_height);
}
