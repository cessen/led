use std;
use std::cell::RefCell;
use std::io;
use std::io::Write;

use super::smallstring::SmallString;
use unicode_width::UnicodeWidthStr;
use unicode_segmentation::UnicodeSegmentation;
use termion;
use termion::screen::AlternateScreen;
use termion::color;
use termion::raw::{IntoRawMode, RawTerminal};

pub(crate) struct Screen {
    out: RefCell<AlternateScreen<RawTerminal<io::Stdout>>>,
    buf: RefCell<Vec<Option<(Style, SmallString)>>>,
    w: usize,
    h: usize,
}

impl Screen {
    pub(crate) fn new() -> Self {
        let (w, h) = termion::terminal_size().unwrap();
        let buf = std::iter::repeat(Some((Style(Color::Black, Color::Black), " ".into())))
            .take(w as usize * h as usize)
            .collect();
        Screen {
            out: RefCell::new(AlternateScreen::from(io::stdout().into_raw_mode().unwrap())),
            buf: RefCell::new(buf),
            w: w as usize,
            h: h as usize,
        }
    }

    pub(crate) fn clear(&self, col: Color) {
        for cell in self.buf.borrow_mut().iter_mut() {
            match *cell {
                Some((ref mut style, ref mut text)) => {
                    *style = Style(col, col);
                    text.clear();
                    text.push_str(" ");
                }
                _ => {
                    *cell = Some((Style(col, col), " ".into()));
                }
            }
        }
    }

    pub(crate) fn resize(&mut self, w: usize, h: usize) {
        self.w = w;
        self.h = h;
        self.buf
            .borrow_mut()
            .resize(w * h, Some((Style(Color::Black, Color::Black), " ".into())));
    }

    pub(crate) fn present(&self) {
        let buf = self.buf.borrow();

        // Double the minimum needed space, because of formatting characters and such.
        let mut tmp_string = String::with_capacity(self.w * self.h * 2);

        // Write everything to the tmp_string first.
        for y in 0..self.h {
            let mut x = 0;
            let mut last_style = Style(Color::Black, Color::Black);
            tmp_string.push_str(&format!("{}", last_style));
            while x < self.w {
                if let Some((style, ref text)) = buf[y * self.w + x] {
                    if style != last_style {
                        tmp_string.push_str(&format!("{}", style));
                        last_style = style;
                    }
                    tmp_string.push_str(text);
                    x += 1;
                } else {
                    x += 1;
                }
            }
        }

        // Then write the tmp_string to screen all at once.
        write!(
            self.out.borrow_mut(),
            "{}{}",
            termion::cursor::Goto(0, 0),
            tmp_string,
        ).unwrap();
        self.out.borrow_mut().flush().unwrap();
    }

    pub(crate) fn draw(&self, x: usize, y: usize, text: &str, style: Style) {
        let mut buf = self.buf.borrow_mut();
        let mut x = x;
        for g in UnicodeSegmentation::graphemes(text, true) {
            let width = UnicodeWidthStr::width(g);
            if width > 0 {
                buf[y * self.w + x] = Some((style, g.into()));
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
            "{}{}{}",
            color::Fg(color::Reset),
            color::Bg(color::Reset),
            termion::clear::All,
        ).unwrap();
        self.show_cursor();
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) enum Color {
    Black,
    Blue,
    Cyan,
    Green,
    LightBlack,
    LightBlue,
    LightCyan,
    LightGreen,
    LightMagenta,
    LightRed,
    LightWhite,
    LightYellow,
    Magenta,
    Red,
    Rgb(u8, u8, u8),
    White,
    Yellow,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) struct Style(pub Color, pub Color); // Fg, Bg

impl std::fmt::Display for Style {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.0 {
            Color::Black => write!(f, "{}", color::Fg(color::Black)),
            Color::Blue => write!(f, "{}", color::Fg(color::Blue)),
            Color::Cyan => write!(f, "{}", color::Fg(color::Cyan)),
            Color::Green => write!(f, "{}", color::Fg(color::Green)),
            Color::LightBlack => write!(f, "{}", color::Fg(color::LightBlack)),
            Color::LightBlue => write!(f, "{}", color::Fg(color::LightBlue)),
            Color::LightCyan => write!(f, "{}", color::Fg(color::LightCyan)),
            Color::LightGreen => write!(f, "{}", color::Fg(color::LightGreen)),
            Color::LightMagenta => write!(f, "{}", color::Fg(color::LightMagenta)),
            Color::LightRed => write!(f, "{}", color::Fg(color::LightRed)),
            Color::LightWhite => write!(f, "{}", color::Fg(color::LightWhite)),
            Color::LightYellow => write!(f, "{}", color::Fg(color::LightYellow)),
            Color::Magenta => write!(f, "{}", color::Fg(color::Magenta)),
            Color::Red => write!(f, "{}", color::Fg(color::Red)),
            Color::Rgb(r, g, b) => write!(f, "{}", color::Fg(color::Rgb(r, g, b))),
            Color::White => write!(f, "{}", color::Fg(color::White)),
            Color::Yellow => write!(f, "{}", color::Fg(color::Yellow)),
        }?;

        match self.1 {
            Color::Black => write!(f, "{}", color::Bg(color::Black)),
            Color::Blue => write!(f, "{}", color::Bg(color::Blue)),
            Color::Cyan => write!(f, "{}", color::Bg(color::Cyan)),
            Color::Green => write!(f, "{}", color::Bg(color::Green)),
            Color::LightBlack => write!(f, "{}", color::Bg(color::LightBlack)),
            Color::LightBlue => write!(f, "{}", color::Bg(color::LightBlue)),
            Color::LightCyan => write!(f, "{}", color::Bg(color::LightCyan)),
            Color::LightGreen => write!(f, "{}", color::Bg(color::LightGreen)),
            Color::LightMagenta => write!(f, "{}", color::Bg(color::LightMagenta)),
            Color::LightRed => write!(f, "{}", color::Bg(color::LightRed)),
            Color::LightWhite => write!(f, "{}", color::Bg(color::LightWhite)),
            Color::LightYellow => write!(f, "{}", color::Bg(color::LightYellow)),
            Color::Magenta => write!(f, "{}", color::Bg(color::Magenta)),
            Color::Red => write!(f, "{}", color::Bg(color::Red)),
            Color::Rgb(r, g, b) => write!(f, "{}", color::Bg(color::Rgb(r, g, b))),
            Color::White => write!(f, "{}", color::Bg(color::White)),
            Color::Yellow => write!(f, "{}", color::Bg(color::Yellow)),
        }
    }
}
