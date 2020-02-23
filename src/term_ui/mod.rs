#![allow(dead_code)]

mod screen;
pub mod smallstring;

use std::time::Duration;

use crossterm::{
    event::{Event, KeyCode, KeyEvent, KeyModifiers},
    style::Color,
};

use crate::{
    editor::Editor,
    formatter::LineFormatter,
    string_utils::{char_count, is_line_ending, line_ending_to_str, LineEnding},
    utils::{digit_count, Timer},
};

use self::screen::{Screen, Style};

const EMPTY_MOD: KeyModifiers = KeyModifiers::empty();
const UPDATE_TICK_MS: u64 = 10;

// Color theme.
// Styles are (FG, BG).
const STYLE_MAIN: Style = Style(
    Color::Rgb {
        r: 0xD0,
        g: 0xD0,
        b: 0xD0,
    },
    Color::Rgb {
        r: 0x30,
        g: 0x30,
        b: 0x30,
    },
);
const STYLE_CURSOR: Style = Style(
    Color::Rgb {
        r: 0x00,
        g: 0x00,
        b: 0x00,
    },
    Color::Rgb {
        r: 0xD0,
        g: 0xD0,
        b: 0xD0,
    },
);
const STYLE_GUTTER_LINE_START: Style = Style(
    Color::Rgb {
        r: 0x78,
        g: 0x78,
        b: 0x78,
    },
    Color::Rgb {
        r: 0x1D,
        g: 0x1D,
        b: 0x1D,
    },
);
const STYLE_GUTTER_LINE_WRAP: Style = Style(
    Color::Rgb {
        r: 0x78,
        g: 0x78,
        b: 0x78,
    },
    Color::Rgb {
        r: 0x27,
        g: 0x27,
        b: 0x27,
    },
);
const COLOR_GUTTER_BAR: Color = Color::Rgb {
    r: 0x18,
    g: 0x18,
    b: 0x18,
};
const STYLE_INFO: Style = Style(
    Color::Rgb {
        r: 0xC0,
        g: 0xC0,
        b: 0xC0,
    },
    Color::Rgb {
        r: 0x14,
        g: 0x14,
        b: 0x14,
    },
);

/// Generalized ui loop.
macro_rules! ui_loop {
    ($term_ui:ident,draw $draw:block,key_press($key:ident) $key_press:block) => {
        let mut stop = false;
        let mut timer = Timer::new();

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
                if crossterm::event::poll(Duration::from_millis(UPDATE_TICK_MS)).unwrap() {
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

                    // If too much time has passed since the last redraw,
                    // break so we can draw if needed.  This keeps an onslaught
                    // of input (e.g. when pasting a large piece of text) from
                    // visually freezing the UI.
                    if timer.elapsed() >= UPDATE_TICK_MS {
                        timer.tick();
                        break;
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
                $term_ui
                    .editor
                    .update_dim($term_ui.height - 1, $term_ui.width);
                $term_ui.editor.formatter.wrap_width = $term_ui.editor.view_dim.1;

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
    editor: Editor,
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
        TermUI::new_from_editor(Editor::new(LineFormatter::new(4)))
    }

    pub fn new_from_editor(ed: Editor) -> TermUI {
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
        self.editor.formatter.wrap_width = self.editor.view_dim.1;
        self.screen.resize(w as usize, h as usize);

        // Start the UI
        ui_loop!(
            self,

            // Draw
            draw {
                self.screen.clear(STYLE_MAIN.1);
                self.draw_editor(&self.editor, (0, 0), (self.height - 1, self.width - 1));
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
                        code: KeyCode::Char('s'),
                        modifiers: KeyModifiers::CONTROL,
                    } => {
                        self.editor.save_if_dirty().expect("For some reason the file couldn't be saved.  Also, TODO: this code path shouldn't panic.");
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
                        code: KeyCode::Up,
                        modifiers: KeyModifiers::CONTROL,
                    } => {
                        self.editor.cursor_up(8);
                    }

                    KeyEvent {
                        code: KeyCode::Down,
                        modifiers: EMPTY_MOD,
                    } => {
                        self.editor.cursor_down(1);
                    }

                    KeyEvent {
                        code: KeyCode::Down,
                        modifiers: KeyModifiers::CONTROL,
                    } => {
                        self.editor.cursor_down(8);
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
                        self.editor.remove_text_behind_cursor(1);
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
        let mut cancel = false;
        let prefix = "Jump to line: ";
        let mut line = String::new();

        ui_loop!(
            self,

            // Draw
            draw {
                self.screen.clear(STYLE_MAIN.1);
                self.draw_editor(&self.editor, (0, 0), (self.height - 1, self.width - 1));
                for i in 0..self.width {
                    self.screen.draw(i, 0, " ", STYLE_INFO);
                }
                self.screen.draw(1, 0, prefix, STYLE_INFO);
                self.screen.draw(
                    prefix.len() + 1,
                    0,
                    &line[..],
                    STYLE_INFO,
                );
                self.screen.set_cursor(prefix.len() + 1, 0);
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
            if let Ok(n) = line.parse::<usize>() {
                self.editor.jump_to_line(n.saturating_sub(1));
            }
        }
    }

    fn draw_editor(&self, editor: &Editor, c1: (usize, usize), c2: (usize, usize)) {
        // Fill in top row with info line color
        for i in c1.1..(c2.1 + 1) {
            self.screen.draw(i, c1.0, " ", STYLE_INFO);
        }

        // Filename and dirty marker
        let filename = editor.file_path.display();
        let dirty_char = if editor.buffer.is_dirty { "*" } else { "" };
        let name = format!("{}{}", filename, dirty_char);
        self.screen.draw(c1.1 + 1, c1.0, &name[..], STYLE_INFO);

        // Percentage position in document
        // TODO: use view instead of cursor for calculation if there is more
        // than one cursor.
        let percentage: usize = if editor.buffer.text.len_chars() > 0 {
            (((editor.buffer.mark_sets[editor.c_msi].main().unwrap().head as f32)
                / (editor.buffer.text.len_chars() as f32))
                * 100.0) as usize
        } else {
            100
        };
        let pstring = format!("{}%", percentage);
        self.screen.draw(
            c2.1.saturating_sub(pstring.len()),
            c1.0,
            &pstring[..],
            STYLE_INFO,
        );

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
            .draw(c2.1.saturating_sub(30), c1.0, &info_line[..], STYLE_INFO);

        // Draw main text editing area
        self.draw_editor_text(editor, (c1.0 + 1, c1.1), c2);
    }

    fn draw_editor_text(&self, editor: &Editor, c1: (usize, usize), c2: (usize, usize)) {
        let view_pos = editor.buffer.mark_sets[editor.v_msi][0].head;
        let cursors = &editor.buffer.mark_sets[editor.c_msi];

        // Calculate all the starting info
        let gutter_width = editor.editor_dim.1 - editor.view_dim.1;
        let blank_gutter = &"                "[..gutter_width - 1];
        let line_index = editor.buffer.text.char_to_line(view_pos);

        let (blocks_iter, char_offset) = editor.formatter.iter(&editor.buffer.text, view_pos);

        let vis_line_offset = blocks_iter.clone().next().unwrap().0.vpos(char_offset);

        let mut screen_line = c1.0 as isize - vis_line_offset as isize;
        let screen_col = c1.1 as isize + gutter_width as isize;

        // Fill in the gutter with the appropriate background
        for y in c1.0..(c2.0 + 1) {
            self.screen
                .draw(c1.1, y, blank_gutter, STYLE_GUTTER_LINE_WRAP);
            self.screen.draw(
                c1.1 + blank_gutter.len() - 1,
                y,
                "▕",
                Style(COLOR_GUTTER_BAR, STYLE_GUTTER_LINE_WRAP.1),
            );
        }

        // Loop through the blocks, printing them to the screen.
        let mut is_first_loop = true;
        let mut line_num = line_index + 1;
        let mut char_index = view_pos - char_offset;
        for (block_vis_iter, is_line_start) in blocks_iter {
            if is_line_start && !is_first_loop {
                line_num += 1;
            }
            is_first_loop = false;

            // Print line number
            if is_line_start {
                let lnx = c1.1;
                let lny = screen_line as usize;
                if lny >= c1.0 && lny <= c2.0 {
                    self.screen.draw(
                        lnx,
                        lny,
                        &format!(
                            "{}{}",
                            &blank_gutter
                                [..(gutter_width - 2 - digit_count(line_num as u32, 10) as usize)],
                            line_num,
                        )[..],
                        STYLE_GUTTER_LINE_START,
                    );
                    self.screen.draw(
                        lnx + blank_gutter.len() - 1,
                        lny,
                        "▕",
                        Style(COLOR_GUTTER_BAR, STYLE_GUTTER_LINE_START.1),
                    );
                }
            }

            // Loop through the graphemes of the block and print them to
            // the screen.
            let mut last_pos_y = 0;
            for (g, (pos_y, pos_x), width) in block_vis_iter {
                // Calculate the cell coordinates at which to draw the grapheme
                if pos_y > last_pos_y {
                    screen_line += 1;
                    last_pos_y = pos_y;
                }
                let px = pos_x as isize + screen_col;
                let py = screen_line;

                // If we're off the bottom, we're done
                if py > c2.0 as isize {
                    return;
                }

                // Draw the grapheme to the screen if it's in bounds
                if (px >= c1.1 as isize) && (py >= c1.0 as isize) && (px <= c2.1 as isize) {
                    // Check if the character is within a cursor
                    let mut at_cursor = false;
                    for c in cursors.iter() {
                        if char_index >= c.range().start && char_index <= c.range().end {
                            at_cursor = true;
                            self.screen.set_cursor(px as usize, py as usize);
                        }
                    }

                    // Actually print the character
                    if is_line_ending(&g) {
                        if at_cursor {
                            self.screen
                                .draw(px as usize, py as usize, " ", STYLE_CURSOR);
                        }
                    } else if g == "\t" {
                        for i in 0..width {
                            let tpx = px as usize + i;
                            if tpx <= c2.1 {
                                self.screen.draw(tpx as usize, py as usize, " ", STYLE_MAIN);
                            }
                        }

                        if at_cursor {
                            self.screen
                                .draw(px as usize, py as usize, " ", STYLE_CURSOR);
                        }
                    } else {
                        if at_cursor {
                            self.screen.draw(px as usize, py as usize, &g, STYLE_CURSOR);
                        } else {
                            self.screen.draw(px as usize, py as usize, &g, STYLE_MAIN);
                        }
                    }
                }

                char_index += char_count(&g);
            }

            screen_line += 1;
        }

        // If we get here, it means we reached the end of the text buffer
        // without going off the bottom of the screen.  So draw the cursor
        // at the end if needed.

        // Check if the character is within a cursor
        let mut at_cursor = false;
        for c in cursors.iter() {
            if char_index >= c.range().start && char_index <= c.range().end {
                at_cursor = true;
            }
        }

        if at_cursor {
            // Calculate the cell coordinates at which to draw the cursor
            let pos_x = editor.formatter.get_horizontal(
                &self.editor.buffer.text,
                self.editor.buffer.text.len_chars(),
            );
            let mut px = pos_x as isize + screen_col;
            let mut py = screen_line - 1;
            if px > c2.1 as isize {
                px = c1.1 as isize + screen_col;
                py += 1;
            }

            if (px >= c1.1 as isize)
                && (py >= c1.0 as isize)
                && (px <= c2.1 as isize)
                && (py <= c2.0 as isize)
            {
                self.screen
                    .draw(px as usize, py as usize, " ", STYLE_CURSOR);
                self.screen.set_cursor(px as usize, py as usize);
            }
        }
    }
}
