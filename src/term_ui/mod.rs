#![allow(dead_code)]

use rustbox;
use rustbox::Color;
use editor::Editor;
use std::char;
use std::time::duration::Duration;
use string_utils::{is_line_ending};
use buffer::line::{line_ending_to_str, LineEnding};
use buffer::line_formatter::{LineFormatter, RoundingBehavior};
use self::formatter::ConsoleLineFormatter;

pub mod formatter;

// Key codes
const K_ENTER: u16 = 13;
const K_TAB: u16 = 9;
const K_SPACE: u16 = 32;
const K_BACKSPACE: u16 = 127;
const K_DELETE: u16 = 65522;
const K_PAGEUP: u16 = 65519;
const K_PAGEDOWN: u16 = 65518;
const K_UP: u16 = 65517;
const K_DOWN: u16 = 65516;
const K_LEFT: u16 = 65515;
const K_RIGHT: u16 = 65514;
const K_ESC: u16 = 27;
const K_CTRL_L: u16 = 12;
const K_CTRL_O: u16 = 15;
const K_CTRL_Q: u16 = 17;
const K_CTRL_S: u16 = 19;
const K_CTRL_Y: u16 = 25;
const K_CTRL_Z: u16 = 26;


pub struct TermUI {
    rb: rustbox::RustBox,
    editor: Editor<ConsoleLineFormatter>,
    width: usize,
    height: usize,
}


impl TermUI {
    pub fn new() -> TermUI {
        let rb = match rustbox::RustBox::init(&[Some(rustbox::InitOption::BufferStderr)]) {
            Ok(rbox) => rbox,
            Err(_) => panic!("Could not create Rustbox instance."),
        };
        let w = rb.width();
        let h = rb.height();
        let mut editor = Editor::new(ConsoleLineFormatter::new(4));
        editor.update_dim(h-1, w);
        
        TermUI {
            rb: rb,
            editor: editor,
            width: w,
            height: h,
        }
    }
    
    pub fn new_from_editor(ed: Editor<ConsoleLineFormatter>) -> TermUI {
        let rb = match rustbox::RustBox::init(&[Some(rustbox::InitOption::BufferStderr)]) {
            Ok(rbox) => rbox,
            Err(_) => panic!("Could not create Rustbox instance."),
        };
        let w = rb.width();
        let h = rb.height();
        let mut editor = ed;
        editor.update_dim(h-1, w);
        
        TermUI {
            rb: rb,
            editor: editor,
            width: w,
            height: h,
        }
    }
    
    pub fn main_ui_loop(&mut self) {
        // Quitting flag
        let mut quit = false;
    
        self.editor.update_dim(self.height-1, self.width);
    
        loop {
            // Draw the editor to screen
            self.rb.clear();
            self.draw_editor(&self.editor, (0, 0), (self.height-1, self.width-1));
            self.rb.present();
            
            
            // Handle events.  We block on the first event, so that the
            // program doesn't loop like crazy, but then continue pulling
            // events in a non-blocking way until we run out of events
            // to handle.
            let mut e = self.rb.poll_event(); // Block until we get an event
            loop {
                match e {
                    Ok(rustbox::Event::KeyEvent(modifier, key, character)) => {
                        //println!("      {} {} {}", modifier, key, character);
                        match key {
                            K_CTRL_Q => {
                                quit = true;
                                break;
                            },
                            
                            K_CTRL_S => {
                                self.editor.save_if_dirty();
                            },
                            
                            K_CTRL_Z => {
                                self.editor.undo();
                            },
                            
                            K_CTRL_Y => {
                                self.editor.redo();
                            },
                            
                            K_CTRL_L => {
                                self.go_to_line_ui_loop();
                            },
                            
                            K_PAGEUP => {
                                self.editor.page_up();
                            },
                            
                            K_PAGEDOWN => {
                                self.editor.page_down();
                            },
                            
                            K_UP => {
                                self.editor.cursor_up(1);
                            },
                            
                            K_DOWN => {
                                self.editor.cursor_down(1);
                            },
                            
                            K_LEFT => {
                                self.editor.cursor_left(1);
                            },
                            
                            K_RIGHT => {
                                self.editor.cursor_right(1);
                            },
                            
                            K_ENTER => {
                                let nl = line_ending_to_str(self.editor.buffer.line_ending_type);
                                self.editor.insert_text_at_cursor(nl);
                            },
                            
                            K_SPACE => {
                                self.editor.insert_text_at_cursor(" ");
                            },
                            
                            K_TAB => {
                                self.editor.insert_tab_at_cursor();
                            },
                            
                            K_BACKSPACE => {
                                self.editor.backspace_at_cursor();
                            },
                            
                            K_DELETE => {
                                self.editor.remove_text_in_front_of_cursor(1);
                            },
                            
                            // Character
                            0 => {
                                if let Option::Some(c) = char::from_u32(character) {
                                    self.editor.insert_text_at_cursor(c.to_string().as_slice());
                                }
                            },
                            
                            _ => {}
                        }
                    },
                    
                    Ok(rustbox::Event::ResizeEvent(w, h)) => {
                        self.width = w as usize;
                        self.height = h as usize;
                        self.editor.update_dim(self.height-1, self.width);
                    },
                    
                    _ => {
                        break;
                    }
                }
    
                e = self.rb.peek_event(Duration::milliseconds(0)); // Get next event (if any)
            }
            
            
            // Quit if quit flag is set
            if quit {
                break;
            }
        }
    }
    
    
    fn go_to_line_ui_loop(&mut self) {
        let foreground = Color::Black;
        let background = Color::Cyan;
    
        let mut cancel = false;
        let mut confirm = false;
        let prefix = "Jump to line: ";
        let mut line = String::new();
        
        loop {
            // Draw the editor to screen
            self.rb.clear();
            self.draw_editor(&self.editor, (0, 0), (self.height-1, self.width-1));
            for i in range(0, self.width) {
                self.rb.print(i, 0, rustbox::RB_NORMAL, foreground, background, " ");
            }
            self.rb.print(1, 0, rustbox::RB_NORMAL, foreground, background, prefix);
            self.rb.print(prefix.len() + 1, 0, rustbox::RB_NORMAL, foreground, background, line.as_slice());
            self.rb.present();
            
            
            // Handle events.  We block on the first event, so that the
            // program doesn't loop like crazy, but then continue pulling
            // events in a non-blocking way until we run out of events
            // to handle.
            let mut e = self.rb.poll_event(); // Block until we get an event
            loop {
                match e {
                    Ok(rustbox::Event::KeyEvent(_, key, character)) => {
                        match key {
                            K_ESC => {
                                cancel = true;
                                break;
                            },
                            
                            K_ENTER => {
                                confirm = true;
                                break;
                            },
                            
                            K_BACKSPACE => {
                                line.pop();
                            },
                            
                            // Character
                            0 => {
                                if let Option::Some(c) = char::from_u32(character) {
                                    if c.is_numeric() {
                                        line.push(c);
                                    }
                                }
                            },
                            
                            _ => {}
                        }
                    },
                    
                    Ok(rustbox::Event::ResizeEvent(w, h)) => {
                        self.width = w as usize;
                        self.height = h as usize;
                        self.editor.update_dim(self.height-1, self.width);
                    },
                    
                    _ => {
                        break;
                    }
                }
    
                e = self.rb.peek_event(Duration::milliseconds(0)); // Get next event (if any)
            }
            
            
            // Cancel if flag is set
            if cancel {
                break;
            }
            
            // Jump to line!
            if confirm {
                if let Some(n) = line.parse() {
                    let n2: usize = n; // Weird work-around: the type of n wasn't being inferred
                    if n2 > 0 {
                        self.editor.jump_to_line(n2-1);
                    }
                    else {
                        self.editor.jump_to_line(0);
                    }
                }
                break;
            }
        }
    }
    
    
    fn draw_editor(&self, editor: &Editor<ConsoleLineFormatter>, c1: (usize, usize), c2: (usize, usize)) {
        let foreground = Color::Black;
        let background = Color::Cyan;
        
        // Fill in top row with info line color
        for i in range(c1.1, c2.1 + 1) {
            self.rb.print(i, c1.0, rustbox::RB_NORMAL, foreground, background, " ");
        }
        
        // Filename and dirty marker
        let filename = editor.file_path.display();
        let dirty_char = if editor.dirty {"*"} else {""};
        let name = format!("{}{}", filename, dirty_char);
        self.rb.print(c1.1 + 1, c1.0, rustbox::RB_NORMAL, foreground, background, name.as_slice());
        
        // Percentage position in document
        // TODO: use view instead of cursor for calculation if there is more
        // than one cursor.
        let percentage: usize = if editor.buffer.grapheme_count() > 0 {
            (((editor.cursors[0].range.0 as f32) / (editor.buffer.grapheme_count() as f32)) * 100.0) as usize
        }
        else {
            100
        };
        let pstring = format!("{}%", percentage);
        self.rb.print(c2.1 - pstring.len(), c1.0, rustbox::RB_NORMAL, foreground, background, pstring.as_slice());
        
        // Text encoding info and tab style
        let nl = match editor.buffer.line_ending_type {
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
        let soft_tabs_str = if editor.soft_tabs {"spaces"} else {"tabs"};
        let info_line = format!("UTF8:{}  {}:{}", nl, soft_tabs_str, editor.soft_tab_width as usize);
        self.rb.print(c2.1 - 30, c1.0, rustbox::RB_NORMAL, foreground, background, info_line.as_slice());

        // Draw main text editing area
        self.draw_editor_text(editor, (c1.0 + 1, c1.1), c2);
    }


    fn draw_editor_text(&self, editor: &Editor<ConsoleLineFormatter>, c1: (usize, usize), c2: (usize, usize)) {
        // Calculate all the starting info
        let editor_corner_index = editor.buffer.v2d_to_index(editor.view_pos, (RoundingBehavior::Floor, RoundingBehavior::Floor));
        let (starting_line, _) = editor.buffer.index_to_line_col(editor_corner_index);
        let mut grapheme_index = editor.buffer.line_col_to_index((starting_line, 0));
        let (vis_starting_line, _) = editor.buffer.index_to_v2d(grapheme_index);

        let mut screen_line = c1.0 as isize + vis_starting_line as isize;
        let screen_col = c1.1 as isize;
        
        let mut line_iter = editor.buffer.line_iter_at_index(starting_line);
        
        for line in line_iter {
            let mut g_iter = editor.buffer.formatter.vis_grapheme_iter(line);
            
            // Loop through the graphemes of the line and print them to
            // the screen.
            for (g, (pos_y, pos_x), width) in g_iter {
                // Calculate the cell coordinates at which to draw the grapheme
                let px = pos_x as isize + screen_col - editor.view_pos.1 as isize;
                let py = pos_y as isize + screen_line - editor.view_pos.0 as isize;
                
                // If we're off the bottom, we're done
                if py > c2.0 as isize {
                    return;
                }
                
                // Draw the grapheme to the screen if it's in bounds
                if (px >= c1.1 as isize) && (py >= c1.0 as isize) && (px <= c2.1 as isize) {
                    // Check if the character is within a cursor
                    let mut at_cursor = false;
                    for c in editor.cursors.iter() {
                        if grapheme_index >= c.range.0 && grapheme_index <= c.range.1 {
                            at_cursor = true;
                        }
                    }
                
                    // Actually print the character
                    if is_line_ending(g) {
                        if at_cursor {
                            self.rb.print(px as usize, py as usize, rustbox::RB_NORMAL, Color::Black, Color::White, " ");
                        }
                    }
                    else if g == "\t" {
                        for i in range(0, width) {
                            let tpx = px as usize + i;
                            if tpx <= c2.1 {
                                self.rb.print(tpx as usize, py as usize, rustbox::RB_NORMAL, Color::White, Color::Black, " ");
                            }
                        }
                        
                        if at_cursor {
                            self.rb.print(px as usize, py as usize, rustbox::RB_NORMAL, Color::Black, Color::White, " ");
                        }
                    }
                    else {
                        if at_cursor {
                            self.rb.print(px as usize, py as usize, rustbox::RB_NORMAL, Color::Black, Color::White, g);
                        }
                        else {
                            self.rb.print(px as usize, py as usize, rustbox::RB_NORMAL, Color::White, Color::Black, g);
                        }
                    }
                }
                
                grapheme_index += 1;
            }
            
            let (dim_y, _) = editor.buffer.formatter.dimensions(line);
            screen_line += dim_y as isize; 
        }
        
        
        
        // If we get here, it means we reached the end of the text buffer
        // without going off the bottom of the screen.  So draw the cursor
        // at the end if needed.
        
        // Check if the character is within a cursor
        let mut at_cursor = false;
        for c in editor.cursors.iter() {
            if grapheme_index >= c.range.0 && grapheme_index <= c.range.1 {
                at_cursor = true;
            }
        }
        
        if at_cursor {
            // Calculate the cell coordinates at which to draw the cursor
            let (pos_y, pos_x) = editor.buffer.index_to_v2d(grapheme_index);
            let px = pos_x as isize + c1.1 as isize - editor.view_pos.1 as isize;
            let py = pos_y as isize + c1.0 as isize - editor.view_pos.0 as isize;
            
            if (px >= c1.1 as isize) && (py >= c1.0 as isize) && (px <= c2.1 as isize) && (py <= c2.0 as isize) {
                self.rb.print(px as usize, py as usize, rustbox::RB_NORMAL, Color::Black, Color::White, " ");
            }
        }
    }
    
    
}