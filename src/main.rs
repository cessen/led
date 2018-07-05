extern crate docopt;
extern crate ropey;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate smallvec;
extern crate termion;
extern crate unicode_segmentation;
extern crate unicode_width;

use docopt::Docopt;
use editor::Editor;
use std::path::Path;
use term_ui::formatter::ConsoleLineFormatter;
use term_ui::TermUI;

mod buffer;
mod editor;
mod formatter;
mod string_utils;
mod term_ui;
mod utils;

// Usage documentation string
static USAGE: &'static str = "
Usage: led [options] [<file>]
       led --help

Options:
    -h, --help  Show this message
";

// Struct for storing command-line arguments
#[derive(Debug, Deserialize)]
struct Args {
    arg_file: Option<String>,
    flag_help: bool,
}

fn main() {
    // Get command-line arguments
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    // Load file, if specified
    let editor = if let Option::Some(s) = args.arg_file {
        Editor::new_from_file(ConsoleLineFormatter::new(4), &Path::new(&s[..]))
    } else {
        Editor::new(ConsoleLineFormatter::new(4))
    };

    // Initialize and start UI
    let mut ui = TermUI::new_from_editor(editor);
    ui.main_ui_loop();
}
