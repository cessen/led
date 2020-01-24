use clap::{App, Arg};
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

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    // Parse command line arguments.
    let args = App::new("Led")
        .version(VERSION)
        .about("A text editor")
        .arg(
            Arg::with_name("file")
                .help("Text file to open")
                .required(false)
                .index(1),
        )
        .get_matches();

    // Load file, if specified
    let editor = if let Some(filepath) = args.value_of("file") {
        Editor::new_from_file(ConsoleLineFormatter::new(4), &Path::new(&filepath[..]))
    } else {
        Editor::new(ConsoleLineFormatter::new(4))
    };

    // Initialize and start UI
    let mut ui = TermUI::new_from_editor(editor);
    ui.main_ui_loop();
}
