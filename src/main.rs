use std::{fs::File, io::BufReader};

use backend::buffer::{Buffer, BufferPath};
use clap::{App, Arg};
use editor::Editor;
use formatter::LineFormatter;
use ropey::Rope;
use term_ui::TermUI;

mod editor;
mod formatter;
mod graphemes;
mod string_utils;
mod term_ui;
mod utils;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> std::io::Result<()> {
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
    let buffer = if let Some(filepath) = args.value_of("file") {
        Buffer::new(
            Rope::from_reader(BufReader::new(File::open(filepath)?))?,
            BufferPath::File(filepath.into()),
        )
    } else {
        Buffer::new("".into(), BufferPath::Temp(0))
    };

    let editor = Editor::new(buffer, LineFormatter::new(4));

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

    Ok(())
}
