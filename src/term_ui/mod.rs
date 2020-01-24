#![allow(dead_code)]

pub mod formatter;
mod screen;
pub mod smallstring;

use std::{cmp::min, time::Duration};

use crossterm::{
    event::{Event, KeyCode, KeyEvent, KeyModifiers},
    style::Color,
};

use crate::{
    editor::Editor,
    formatter::{block_index_and_offset, LineFormatter, LINE_BLOCK_LENGTH},
    string_utils::{line_ending_to_str, rope_slice_is_line_ending, LineEnding},
    utils::{digit_count, RopeGraphemes},
};

use self::{
    formatter::ConsoleLineFormatter,
    screen::{Screen, Style},
};

const EMPTY_MOD: KeyModifiers = KeyModifiers::empty();

/// Generalized ui loop.
macro_rules! ui_loop {
    ($term_ui:ident,draw $draw:block,key_press($key:ident) $key_press:block) => {
        let mut stop = false;

        // Draw the editor to screen for the first time
        {
            $draw
        };
        $term_ui.screen.present();

        // UI loop
        loop {
            let mut should_redraw = false;

            // Handle input.
            // Doing this as a polled loop isn't necessary in the current
            // implementation, but it will be useful in the future when we may
            // want to re-draw on e.g. async syntax highlighting updates, or
            // update based on a file being modified outside our process.
            loop {
                if crossterm::event::poll(Duration::from_millis(5)).unwrap() {
                    match crossterm::event::read().unwrap() {
                        Event::Key($key) => {
                            let (status, state_changed) = || -> (LoopStatus, bool) { $key_press }();
                            should_redraw |= state_changed;
                            if status == LoopStatus::Done {
                                stop = true;
                                break;
                            }
                        }

                        Event::Mouse(_) => {
                            break;
                        }

                        Event::Resize(w, h) => {
                            $term_ui.width = w as usize;
                            $term_ui.height = h as usize;
                            $term_ui.screen.resize(w as usize, h as usize);
                            should_redraw = true;
                            break;
                        }
                    }
                } else {
                    break;
                }
            }

            // Check if we're done
            if stop || $term_ui.quit {
                break;
            }

            // Draw the editor to screen
            if should_redraw {
                // Make sure display dimensions are up-to-date.
                $term_ui.editor.update_dim($term_ui.height, $term_ui.width);
                $term_ui
                    .editor
                    .formatter
                    .set_wrap_width($term_ui.editor.view_dim.1);

                // Draw!
                {
                    $draw
                };
                $term_ui.screen.present();
            }
        }
    };
}

pub struct TermUI {
    screen: Screen,
    editor: Editor<ConsoleLineFormatter>,
    width: usize,
    height: usize,
    quit: bool,
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum LoopStatus {
    Done,
    Continue,
}

impl TermUI {
    pub fn new() -> TermUI {
        TermUI::new_from_editor(Editor::new(ConsoleLineFormatter::new(4)))
    }

    pub fn new_from_editor(ed: Editor<ConsoleLineFormatter>) -> TermUI {
        let (w, h) = crossterm::terminal::size().unwrap();
        let mut editor = ed;
        editor.update_dim(h as usize - 1, w as usize);

        TermUI {
            screen: Screen::new(),
            editor: editor,
            width: w as usize,
            height: h as usize,
            quit: false,
        }
    }

    pub fn main_ui_loop(&mut self) {
        // Hide cursor
        self.screen.hide_cursor();

        // Set terminal size info
        let (w, h) = crossterm::terminal::size().unwrap();
        self.width = w as usize;
        self.height = h as usize;
        self.editor.update_dim(self.height - 1, self.width);
        self.editor.formatter.set_wrap_width(self.editor.view_dim.1);
        self.screen.resize(w as usize, h as usize);

        // Start the UI
        ui_loop!(
            self,

            // Draw
            draw {
                self.screen.clear(Color::Black);
                self.draw_editor(&self.editor, (0, 0), (self.height - 1, self.width - 1));
            },

            // Handle input
            key_press(key) {
                let mut state_changed = true;
                match key {
                    KeyEvent {
                        code: KeyCode::Char('q'),
                        // modifiers: EMPTY_MOD,
                        modifiers: KeyModifiers::CONTROL,
                    } => {
                        self.quit = true;
                        return (LoopStatus::Done, true);
                    }

                    KeyEvent {
                        code: KeyCode::Char('s'),
                        modifiers: KeyModifiers::CONTROL,
                    } => {
                        self.editor.save_if_dirty();
                    }

                    KeyEvent {
                        code: KeyCode::Char('z'),
                        modifiers: KeyModifiers::CONTROL,
                    } => {
                        self.editor.undo();
                    }

                    KeyEvent {
                        code: KeyCode::Char('y'),
                        modifiers: KeyModifiers::CONTROL,
                    } => {
                        self.editor.redo();
                    }

                    KeyEvent {
                        code: KeyCode::Char('l'),
                        modifiers: KeyModifiers::CONTROL,
                    } => {
                        self.go_to_line_ui_loop();
                    }

                    KeyEvent {
                        code: KeyCode::PageUp,
                        modifiers: EMPTY_MOD,
                    } => {
                        self.editor.page_up();
                    }

                    KeyEvent {
                        code: KeyCode::PageDown,
                        modifiers: EMPTY_MOD,
                    } => {
                        self.editor.page_down();
                    }

                    KeyEvent {
                        code: KeyCode::Up,
                        modifiers: EMPTY_MOD,
                    } => {
                        self.editor.cursor_up(1);
                    }

                    KeyEvent {
                        code: KeyCode::Down,
                        modifiers: EMPTY_MOD,
                    } => {
                        self.editor.cursor_down(1);
                    }

                    KeyEvent {
                        code: KeyCode::Left,
                        modifiers: EMPTY_MOD,
                    } => {
                        self.editor.cursor_left(1);
                    }

                    KeyEvent {
                        code: KeyCode::Right,
                        modifiers: EMPTY_MOD,
                    } => {
                        self.editor.cursor_right(1);
                    }

                    KeyEvent {
                        code: KeyCode::Enter,
                        modifiers: EMPTY_MOD,
                    } => {
                        let nl = line_ending_to_str(self.editor.line_ending_type);
                        self.editor.insert_text_at_cursor(nl);
                    }

                    KeyEvent {
                        code: KeyCode::Tab,
                        modifiers: EMPTY_MOD,
                    } => {
                        self.editor.insert_tab_at_cursor();
                    }

                    KeyEvent {
                        code: KeyCode::Backspace,
                        modifiers: EMPTY_MOD,
                    } => {
                        self.editor.backspace_at_cursor();
                    }

                    KeyEvent {
                        code: KeyCode::Delete,
                        modifiers: EMPTY_MOD,
                    } => {
                        self.editor.remove_text_in_front_of_cursor(1);
                    }

                    // Character
                    KeyEvent {
                        code: KeyCode::Char(c),
                        modifiers: EMPTY_MOD,
                    } => {
                        self.editor.insert_text_at_cursor(&c.to_string()[..]);
                    }

                    _ => {
                        state_changed = false;
                    }
                }

                (LoopStatus::Continue, state_changed)
            }
        );
    }

    fn go_to_line_ui_loop(&mut self) {
        let style = Style(Color::Black, Color::Cyan);

        let mut cancel = false;
        let prefix = "Jump to line: ";
        let mut line = String::new();

        ui_loop!(
            self,

            // Draw
            draw {
                self.screen.clear(Color::Black);
                self.draw_editor(&self.editor, (0, 0), (self.height - 1, self.width - 1));
                for i in 0..self.width {
                    self.screen.draw(i, 0, " ", style);
                }
                self.screen.draw(1, 0, prefix, style);
                self.screen.draw(
                    prefix.len() + 1,
                    0,
                    &line[..],
                    style,
                );
            },

            // Handle input
            key_press(key) {
                let mut state_changed = true;
                match key {
                    KeyEvent {
                        code: KeyCode::Char('q'),
                        modifiers: KeyModifiers::CONTROL,
                    } => {
                        self.quit = true;
                        return (LoopStatus::Done, true);
                    }

                    KeyEvent {
                        code: KeyCode::Esc,
                        modifiers: EMPTY_MOD,
                    } => {
                        cancel = true;
                        return (LoopStatus::Done, true);
                    }

                    KeyEvent {
                        code: KeyCode::Enter,
                        modifiers: EMPTY_MOD,
                    } => {
                        return (LoopStatus::Done, true);
                    }

                    KeyEvent {
                        code: KeyCode::Backspace,
                        modifiers: EMPTY_MOD,
                    } => {
                        line.pop();
                    }

                    // Character
                    KeyEvent {
                        code: KeyCode::Char(c),
                        modifiers: EMPTY_MOD,
                    } => {
                        if c.is_numeric() {
                            line.push(c);
                        }
                    }

                    _ => {
                        state_changed = false;
                    }
                }

                return (LoopStatus::Continue, state_changed);
            }
        );

        // Jump to line!
        if !cancel {
            if let Ok(n) = line.parse() {
                let n2: usize = n; // Weird work-around: the type of n wasn't being inferred
                if n2 > 0 {
                    self.editor.jump_to_line(n2 - 1);
                } else {
                    self.editor.jump_to_line(0);
                }
            }
        }
    }

    fn draw_editor(
        &self,
        editor: &Editor<ConsoleLineFormatter>,
        c1: (usize, usize),
        c2: (usize, usize),
    ) {
        let style = Style(Color::Black, Color::Cyan);

        // Fill in top row with info line color
        for i in c1.1..(c2.1 + 1) {
            self.screen.draw(i, c1.0, " ", style);
        }

        // Filename and dirty marker
        let filename = editor.file_path.display();
        let dirty_char = if editor.dirty { "*" } else { "" };
        let name = format!("{}{}", filename, dirty_char);
        self.screen.draw(c1.1 + 1, c1.0, &name[..], style);

        // Percentage position in document
        // TODO: use view instead of cursor for calculation if there is more
        // than one cursor.
        let percentage: usize = if editor.buffer.char_count() > 0 {
            (((editor.cursors[0].range.0 as f32) / (editor.buffer.char_count() as f32)) * 100.0)
                as usize
        } else {
            100
        };
        let pstring = format!("{}%", percentage);
        self.screen
            .draw(c2.1 - pstring.len().min(c2.1), c1.0, &pstring[..], style);

        // Text encoding info and tab style
        let nl = match editor.line_ending_type {
            LineEnding::None => "None",
            LineEnding::CRLF => "CRLF",
            LineEnding::LF => "LF",
            LineEnding::VT => "VT",
            LineEnding::FF => "FF",
            LineEnding::CR => "CR",
            LineEnding::NEL => "NEL",
            LineEnding::LS => "LS",
            LineEnding::PS => "PS",
        };
        let soft_tabs_str = if editor.soft_tabs { "spaces" } else { "tabs" };
        let info_line = format!(
            "UTF8:{}  {}:{}",
            nl, soft_tabs_str, editor.soft_tab_width as usize
        );
        self.screen
            .draw(c2.1 - 30.min(c2.1), c1.0, &info_line[..], style);

        // Draw main text editing area
        self.draw_editor_text(editor, (c1.0 + 1, c1.1), c2);
    }

    fn draw_editor_text(
        &self,
        editor: &Editor<ConsoleLineFormatter>,
        c1: (usize, usize),
        c2: (usize, usize),
    ) {
        // Calculate all the starting info
        let gutter_width = editor.editor_dim.1 - editor.view_dim.1;
        let (line_index, col_i) = editor.buffer.index_to_line_col(editor.view_pos.0);
        let (mut line_block_index, _) = block_index_and_offset(col_i);
        let mut char_index = editor
            .buffer
            .line_col_to_index((line_index, line_block_index * LINE_BLOCK_LENGTH));
        let temp_line = editor.buffer.get_line(line_index);
        let (vis_line_offset, _) = editor.formatter.index_to_v2d(
            RopeGraphemes::new(&temp_line.slice(
                (line_block_index * LINE_BLOCK_LENGTH)
                    ..min(
                        temp_line.len_chars(),
                        (line_block_index + 1) * LINE_BLOCK_LENGTH,
                    ),
            )),
            editor.view_pos.0 - char_index,
        );

        let mut screen_line = c1.0 as isize - vis_line_offset as isize;
        let screen_col = c1.1 as isize + gutter_width as isize;

        // Fill in the gutter with the appropriate background
        for y in c1.0..(c2.0 + 1) {
            for x in c1.1..(c1.1 + gutter_width - 1) {
                self.screen
                    .draw(x, y, " ", Style(Color::White, Color::Blue));
            }
        }

        let mut line_num = line_index + 1;
        for line in editor.buffer.line_iter_at_index(line_index) {
            // Print line number
            if line_block_index == 0 {
                let lnx = c1.1 + (gutter_width - 1 - digit_count(line_num as u32, 10) as usize);
                let lny = screen_line as usize;
                if lny >= c1.0 && lny <= c2.0 {
                    self.screen.draw(
                        lnx,
                        lny,
                        &format!("{}", line_num)[..],
                        Style(Color::White, Color::Blue),
                    );
                }
            }

            // Loop through the graphemes of the line and print them to
            // the screen.
            let mut line_g_index: usize = 0;
            let mut last_pos_y = 0;
            let mut lines_traversed: usize = 0;
            let line_len = line.len_chars();
            let mut g_iter = editor.formatter.iter(RopeGraphemes::new(
                &line.slice((line_block_index * LINE_BLOCK_LENGTH)..line_len),
            ));

            loop {
                if let Some((g, (pos_y, pos_x), width)) = g_iter.next() {
                    if last_pos_y != pos_y {
                        if last_pos_y < pos_y {
                            lines_traversed += pos_y - last_pos_y;
                        }
                        last_pos_y = pos_y;
                    }
                    // Calculate the cell coordinates at which to draw the grapheme
                    let px = pos_x as isize + screen_col - editor.view_pos.1 as isize;
                    let py = lines_traversed as isize + screen_line;

                    // If we're off the bottom, we're done
                    if py > c2.0 as isize {
                        return;
                    }

                    // Draw the grapheme to the screen if it's in bounds
                    if (px >= c1.1 as isize) && (py >= c1.0 as isize) && (px <= c2.1 as isize) {
                        // Check if the character is within a cursor
                        let mut at_cursor = false;
                        for c in editor.cursors.iter() {
                            if char_index >= c.range.0 && char_index <= c.range.1 {
                                at_cursor = true;
                            }
                        }

                        // Actually print the character
                        if rope_slice_is_line_ending(&g) {
                            if at_cursor {
                                self.screen.draw(
                                    px as usize,
                                    py as usize,
                                    " ",
                                    Style(Color::Black, Color::White),
                                );
                            }
                        } else if g == "\t" {
                            for i in 0..width {
                                let tpx = px as usize + i;
                                if tpx <= c2.1 {
                                    self.screen.draw(
                                        tpx as usize,
                                        py as usize,
                                        " ",
                                        Style(Color::White, Color::Black),
                                    );
                                }
                            }

                            if at_cursor {
                                self.screen.draw(
                                    px as usize,
                                    py as usize,
                                    " ",
                                    Style(Color::Black, Color::White),
                                );
                            }
                        } else {
                            if at_cursor {
                                self.screen.draw_rope_slice(
                                    px as usize,
                                    py as usize,
                                    &g,
                                    Style(Color::Black, Color::White),
                                );
                            } else {
                                self.screen.draw_rope_slice(
                                    px as usize,
                                    py as usize,
                                    &g,
                                    Style(Color::White, Color::Black),
                                );
                            }
                        }
                    }

                    char_index += g.chars().count();
                    line_g_index += 1;
                } else {
                    break;
                }

                if line_g_index >= LINE_BLOCK_LENGTH {
                    line_block_index += 1;
                    line_g_index = 0;
                    let line_len = line.len_chars();
                    g_iter = editor.formatter.iter(RopeGraphemes::new(
                        &line.slice((line_block_index * LINE_BLOCK_LENGTH)..line_len),
                    ));
                    lines_traversed += 1;
                }
            }

            line_block_index = 0;
            screen_line += lines_traversed as isize + 1;
            line_num += 1;
        }

        // If we get here, it means we reached the end of the text buffer
        // without going off the bottom of the screen.  So draw the cursor
        // at the end if needed.

        // Check if the character is within a cursor
        let mut at_cursor = false;
        for c in editor.cursors.iter() {
            if char_index >= c.range.0 && char_index <= c.range.1 {
                at_cursor = true;
            }
        }

        if at_cursor {
            // Calculate the cell coordinates at which to draw the cursor
            let pos_x = editor
                .formatter
                .index_to_horizontal_v2d(&self.editor.buffer, self.editor.buffer.char_count());
            let px = pos_x as isize + screen_col - editor.view_pos.1 as isize;
            let py = screen_line - 1;

            if (px >= c1.1 as isize)
                && (py >= c1.0 as isize)
                && (px <= c2.1 as isize)
                && (py <= c2.0 as isize)
            {
                self.screen.draw(
                    px as usize,
                    py as usize,
                    " ",
                    Style(Color::Black, Color::White),
                );
            }
        }
    }
}
