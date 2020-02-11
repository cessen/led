use std;
use std::cell::{Cell, RefCell};
use std::io;
use std::io::{BufWriter, Write};

use crossterm::{self, execute, queue};
use unicode_segmentation::UnicodeSegmentation;

use crate::utils::grapheme_width;

use super::smallstring::SmallString;

pub(crate) struct Screen {
    out: RefCell<BufWriter<io::Stdout>>,
    buf: RefCell<Vec<Option<(Style, SmallString)>>>,
    main_cursor: Cell<(u16, u16)>,
    w: usize,
    h: usize,
}

impl Screen {
    pub(crate) fn new() -> Self {
        let mut out = BufWriter::with_capacity(1 << 14, io::stdout());
        execute!(out, crossterm::terminal::EnterAlternateScreen).unwrap();
        out.flush().unwrap();
        crossterm::terminal::enable_raw_mode().unwrap();

        let (w, h) = crossterm::terminal::size().unwrap();
        let buf = std::iter::repeat(Some((
            Style(
                crossterm::style::Color::White,
                crossterm::style::Color::Black,
            ),
            " ".into(),
        )))
        .take(w as usize * h as usize)
        .collect();

        Screen {
            out: RefCell::new(out),
            buf: RefCell::new(buf),
            main_cursor: Cell::new((0, 0)),
            w: w as usize,
            h: h as usize,
        }
    }

    pub(crate) fn clear(&self, col: crossterm::style::Color) {
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
        self.buf.borrow_mut().resize(
            w * h,
            Some((
                Style(
                    crossterm::style::Color::White,
                    crossterm::style::Color::Black,
                ),
                " ".into(),
            )),
        );
    }

    pub(crate) fn present(&self) {
        let mut out = self.out.borrow_mut();
        let buf = self.buf.borrow();

        let mut last_style = Style(
            crossterm::style::Color::White,
            crossterm::style::Color::Black,
        );
        queue!(
            out,
            crossterm::style::SetForegroundColor(last_style.0),
            crossterm::style::SetBackgroundColor(last_style.1),
        )
        .unwrap();

        // Write everything to the buffered output.
        for y in 0..self.h {
            let mut x = 0;
            queue!(out, crossterm::cursor::MoveTo(0, y as u16)).unwrap();
            while x < self.w {
                if let Some((style, ref text)) = buf[y * self.w + x] {
                    if style != last_style {
                        queue!(
                            out,
                            crossterm::style::SetForegroundColor(style.0),
                            crossterm::style::SetBackgroundColor(style.1),
                        )
                        .unwrap();
                        last_style = style;
                    }
                    write!(out, "{}", text).unwrap();
                    x += 1;
                } else {
                    x += 1;
                }
            }
        }

        let cursor_pos = self.main_cursor.get();
        queue!(out, crossterm::cursor::MoveTo(cursor_pos.0, cursor_pos.1)).unwrap();
        self.main_cursor.set((0, 0));

        // Make sure everything is written out from the buffer.
        out.flush().unwrap();
    }

    pub(crate) fn set_cursor(&self, x: usize, y: usize) {
        self.main_cursor.set((
            x.min(self.w.saturating_sub(1)) as u16,
            y.min(self.h.saturating_sub(1)) as u16,
        ));
    }

    pub(crate) fn draw(&self, x: usize, y: usize, text: &str, style: Style) {
        if y < self.h {
            let mut buf = self.buf.borrow_mut();
            let mut x = x;
            for g in UnicodeSegmentation::graphemes(text, true) {
                let width = grapheme_width(g);
                if width > 0 {
                    if x < self.w {
                        buf[y * self.w + x] = Some((style, g.into()));
                    }
                    x += 1;
                    for _ in 0..(width - 1) {
                        if x < self.w {
                            buf[y * self.w + x] = None;
                        }
                        x += 1;
                    }
                }
            }
        }
    }

    pub(crate) fn hide_cursor(&self) {
        let mut out = self.out.borrow_mut();
        execute!(out, crossterm::cursor::Hide).unwrap();
        out.flush().unwrap();
    }

    pub(crate) fn show_cursor(&self) {
        let mut out = self.out.borrow_mut();
        execute!(out, crossterm::cursor::Show).unwrap();
        out.flush().unwrap();
    }
}

impl Drop for Screen {
    fn drop(&mut self) {
        crossterm::terminal::disable_raw_mode().unwrap();
        let mut out = self.out.borrow_mut();
        execute!(
            out,
            crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
            crossterm::style::ResetColor,
            // crossterm::style::Attribute::Reset,
            crossterm::terminal::LeaveAlternateScreen,
            crossterm::cursor::Show,
        )
        .unwrap();
        out.flush().unwrap();
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) struct Style(pub crossterm::style::Color, pub crossterm::style::Color); // Fg, Bg
