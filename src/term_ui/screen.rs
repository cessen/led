use std::cell::RefCell;
use std::io;
use std::io::Write;

use termion;
use termion::screen::AlternateScreen;
use termion::color;
use termion::raw::{IntoRawMode, RawTerminal};

pub(crate) struct Screen {
    out: RefCell<AlternateScreen<RawTerminal<io::Stdout>>>,
}

impl Screen {
    pub(crate) fn new() -> Self {
        Screen {
            out: RefCell::new(AlternateScreen::from(io::stdout().into_raw_mode().unwrap())),
        }
    }

    pub(crate) fn clear(&self) {
        write!(
            self.out.borrow_mut(),
            "{}{}",
            color::Bg(color::Black),
            termion::clear::All
        ).unwrap();
    }

    pub(crate) fn present(&self) {
        self.out.borrow_mut().flush().unwrap();
    }

    pub(crate) fn draw<C1: color::Color, C2: color::Color>(
        &self,
        x: usize,
        y: usize,
        text: &str,
        fg: C1,
        bg: C2,
    ) {
        write!(
            self.out.borrow_mut(),
            "{}{}{}{}",
            termion::cursor::Goto((x + 1) as u16, (y + 1) as u16),
            color::Fg(fg),
            color::Bg(bg),
            text
        ).unwrap();
    }

    pub(crate) fn hide_cursor(&self) {
        write!(self.out.borrow_mut(), "{}", termion::cursor::Hide).unwrap();
    }

    pub(crate) fn show_cursor(&self) {
        write!(self.out.borrow_mut(), "{}", termion::cursor::Show).unwrap();
    }
}

impl Drop for Screen {
    fn drop(&mut self) {
        write!(
            self.out.borrow_mut(),
            "{}{}",
            color::Fg(color::Reset),
            color::Bg(color::Reset)
        ).unwrap();
        self.clear();
        self.show_cursor();
    }
}
