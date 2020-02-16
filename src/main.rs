use std::{io::Write, path::Path};

use clap::{App, Arg};
use editor::Editor;
use formatter::LineFormatter;
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
        Editor::new_from_file(LineFormatter::new(4), &Path::new(&filepath[..]))
    } else {
        Editor::new(LineFormatter::new(4))
    };

    // Holds stderr output in an internal buffer, and prints it when dropped.
    // This keeps stderr from being swallowed by the TUI.
    let stderr_hold = gag::Hold::stderr().unwrap();

    // Initialize and start UI.
    let exec_result = std::panic::catch_unwind(|| {
        let mut ui = TermUI::new_from_editor(editor);
        ui.main_ui_loop();
    });

    // If we panicked, ensure that we've exited from raw mode and the alternate
    // screen before printing the error and resuming the panic.
    if let Err(e) = exec_result {
        // Exit raw alt screen.
        crossterm::terminal::disable_raw_mode().unwrap();
        crossterm::execute!(std::io::stdout(), crossterm::terminal::LeaveAlternateScreen).unwrap();

        // Print captured stderr.
        drop(stderr_hold);

        // Resume panic unwind.
        std::panic::resume_unwind(e);
    }
}
