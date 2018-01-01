use std;
use std::cell::RefCell;
use std::io;
use std::io::Write;

use unicode_width::UnicodeWidthStr;
use unicode_segmentation::UnicodeSegmentation;
use termion;
use termion::screen::AlternateScreen;
use termion::color;
use termion::raw::{IntoRawMode, RawTerminal};

pub(crate) struct Screen {
    out: RefCell<AlternateScreen<RawTerminal<io::Stdout>>>,
    buf: RefCell<Vec<Option<String>>>,
    w: usize,
    h: usize,
}

impl Screen {
    pub(crate) fn new() -> Self {
        let (w, h) = termion::terminal_size().unwrap();
        let buf = std::iter::repeat(Some(format!(
            "{}{} ",
            color::Fg(color::Black),
            color::Bg(color::Black)
        ))).take(w as usize * h as usize)
            .collect();
        Screen {
            out: RefCell::new(AlternateScreen::from(io::stdout().into_raw_mode().unwrap())),
            buf: RefCell::new(buf),
            w: w as usize,
            h: h as usize,
        }
    }

    pub(crate) fn clear(&self) {
        for cell in self.buf.borrow_mut().iter_mut() {
            match *cell {
                Some(ref mut text) => {
                    text.clear();
                    text.push_str(&format!(
                        "{}{} ",
                        color::Fg(color::Black),
                        color::Bg(color::Black)
                    ));
                }
                _ => {
                    *cell = Some(format!(
                        "{}{} ",
                        color::Fg(color::Black),
                        color::Bg(color::Black)
                    ));
                }
            }
        }
    }

    pub(crate) fn resize(&mut self, w: usize, h: usize) {
        self.w = w;
        self.h = h;
        self.buf.borrow_mut().resize(
            w * h,
            Some(format!(
                "{}{} ",
                color::Fg(color::Black),
                color::Bg(color::Black)
            )),
        );
    }

    pub(crate) fn present(&self) {
        let buf = self.buf.borrow();
        for y in 0..self.h {
            for x in 0..self.w {
                if let Some(ref cell) = buf[y * self.w + x] {
                    write!(
                        self.out.borrow_mut(),
                        "{}{}",
                        termion::cursor::Goto((x + 1) as u16, (y + 1) as u16),
                        cell
                    ).unwrap();
                }
            }
        }
        self.out.borrow_mut().flush().unwrap();
    }

    pub(crate) fn draw<C1: color::Color + Copy, C2: color::Color + Copy>(
        &self,
        x: usize,
        y: usize,
        text: &str,
        fg: C1,
        bg: C2,
    ) {
        let mut buf = self.buf.borrow_mut();
        let mut x = x;
        for g in UnicodeSegmentation::graphemes(text, true) {
            let width = UnicodeWidthStr::width(g);
            if width > 0 {
                buf[y * self.w + x] = Some(format!("{}{}{}", color::Fg(fg), color::Bg(bg), g));
                x += 1;
                for _ in 0..(width - 1) {
                    buf[y * self.w + x] = None;
                    x += 1;
                }
            }
        }
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
