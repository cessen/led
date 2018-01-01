extern crate docopt;
extern crate ropey;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate termion;
extern crate unicode_segmentation;
extern crate unicode_width;

use std::path::Path;
use docopt::Docopt;
use editor::Editor;
use term_ui::TermUI;
use term_ui::formatter::ConsoleLineFormatter;

mod string_utils;
mod utils;
mod buffer;
mod formatter;
mod editor;
mod term_ui;

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
    let mut stdin = std::io::stdin();
    let mut ui = TermUI::new_from_editor(&mut stdin, editor);
    ui.main_ui_loop();
}
