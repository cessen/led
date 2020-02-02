use std::{
    io::{Read, Write},
    path::Path,
};

use clap::{App, Arg};
use editor::Editor;
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

    // Redirect stderr to an internal buffer, so we can print it after exiting.
    let stderr_buf = gag::BufferRedirect::stderr().unwrap();

    // Initialize and start UI
    let exec_result = std::panic::catch_unwind(|| {
        let mut ui = TermUI::new_from_editor(editor);
        ui.main_ui_loop();
    });

    // Check for panics.  If we did panic, exit from raw mode and the alternate
    // screen before propagating the panic, so that the panic printout actually
    // goes to a visible and scrollable screen.
    match exec_result {
        Ok(_) => {
            // Print captured stderr.
            let mut msg = String::new();
            stderr_buf.into_inner().read_to_string(&mut msg).unwrap();
            eprint!("{}", msg);
        }
        Err(e) => {
            // Exit raw alt screen.
            crossterm::terminal::disable_raw_mode().unwrap();
            crossterm::execute!(std::io::stdout(), crossterm::terminal::LeaveAlternateScreen)
                .unwrap();

            // Print captured stderr.
            let mut msg = String::new();
            stderr_buf.into_inner().read_to_string(&mut msg).unwrap();
            eprint!("{}", msg);

            // Resume panic unwind.
            std::panic::resume_unwind(e);
        }
    }
}
