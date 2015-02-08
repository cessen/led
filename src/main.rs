#![feature(core)]
#![feature(io)]
#![feature(collections)]
#![feature(path)]
#![feature(std_misc)]

extern crate rustbox;
extern crate docopt;
extern crate "rustc-serialize" as rustc_serialize;
//extern crate freetype;
//extern crate sdl2;

use std::old_path::Path;
use docopt::Docopt;
use editor::Editor;
use term_ui::TermUI;
use term_ui::formatter::ConsoleLineFormatter;
//use gui::GUI;
//use gui::formatter::GUILineFormatter;

mod string_utils;
mod buffer;
mod formatter;
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
#[derive(RustcDecodable, Debug)]
    struct Args {
    arg_file: Option<String>,
    flag_gui: bool,
    flag_help: bool,
}




fn main() {
    // Get command-line arguments
    let args: Args = Docopt::new(USAGE).and_then(|d| d.decode()).unwrap_or_else(|e| e.exit());
    
    
    //Initialize and start UI
    if args.flag_gui {
        // // Load file, if specified    
        // let editor = if let Option::Some(s) = args.arg_file {
        //     Editor::new_from_file(GUILineFormatter::new(4), &Path::new(&s[]))
        // }
        // else {
        //     Editor::new(GUILineFormatter::new(4))
        // };
        // 
        // // GUI
        // sdl2::init(sdl2::INIT_VIDEO);
        // let mut ui = GUI::new_from_editor(editor);
        // ui.main_ui_loop();
        // sdl2::quit();
    }
    else {
        // Load file, if specified    
        let editor = if let Option::Some(s) = args.arg_file {
            Editor::new_from_file(ConsoleLineFormatter::new(4), &Path::new(&s[]))
        }
        else {
            Editor::new(ConsoleLineFormatter::new(4))
        };
        
        // Console UI
        let mut ui = TermUI::new_from_editor(editor);
        ui.main_ui_loop();
    }
}
