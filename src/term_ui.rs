#![allow(dead_code)]

use rustbox;
use rustbox::Color;
use editor::Editor;
use std::char;
use std::time::duration::Duration;
use string_utils::{is_line_ending};
use buffer::line::{line_ending_to_str, LineEnding};

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
    editor: Editor,
    width: usize,
    height: usize,
}


impl TermUI {
    pub fn new() -> TermUI {
        let rb = rustbox::RustBox::init(&[Some(rustbox::InitOption::BufferStderr)]).unwrap();
        let w = rb.width();
        let h = rb.height();
        let mut editor = Editor::new();
        editor.update_dim(h-1, w);
        
        TermUI {
            rb: rb,
            editor: editor,
            width: w,
            height: h,
        }
    }
    
    pub fn new_from_editor(ed: Editor) -> TermUI {
        let rb = rustbox::RustBox::init(&[Some(rustbox::InitOption::BufferStderr)]).unwrap();
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
                    Ok(rustbox::Event::KeyEvent(modifier, key, character)) => {
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
    
    
    fn draw_editor(&self, editor: &Editor, c1: (usize, usize), c2: (usize, usize)) {
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
        let percentage: usize = if editor.buffer.grapheme_count() > 0 {
            (((editor.cursor.range.0 as f32) / (editor.buffer.grapheme_count() as f32)) * 100.0) as usize
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
        let info_line = format!("UTF8:{}  {}:{}", nl, soft_tabs_str, editor.tab_width);
        self.rb.print(c2.1 - 30, c1.0, rustbox::RB_NORMAL, foreground, background, info_line.as_slice());

        // Draw main text editing area
        self.draw_editor_text(editor, (c1.0 + 1, c1.1), c2);
    }


    fn draw_editor_text(&self, editor: &Editor, c1: (usize, usize), c2: (usize, usize)) {
        let mut line_iter = editor.buffer.line_iter_at_index(editor.view_pos.0);
        
        let mut grapheme_index;
        
        let mut vis_line_num = editor.view_pos.0;
        let mut vis_col_num = editor.view_pos.1;
        
        let mut print_line_num = c1.0;
        let mut print_col_num = c1.1;
        
        let max_print_line = c2.0;
        let max_print_col = c2.1;
        
        loop {
            if let Some(line) = line_iter.next() {
                let mut g_iter = line.grapheme_vis_iter(editor.tab_width);
                let excess = g_iter.skip_vis_positions(editor.view_pos.1);
                
                vis_col_num += excess;
                print_col_num += excess;
                
                grapheme_index = editor.buffer.v2d_to_index((vis_line_num, vis_col_num), editor.tab_width);
                
                for (g, pos, width) in g_iter {
                    print_col_num = pos - editor.view_pos.1;
                    
                    if is_line_ending(g) {
                        if grapheme_index == editor.cursor.range.0 {
                            self.rb.print(print_col_num, print_line_num, rustbox::RB_NORMAL, Color::Black, Color::White, " ");
                        }
                    }
                    else if g == "\t" {
                        for i in range(print_col_num, print_col_num + width) {
                            self.rb.print(i, print_line_num, rustbox::RB_NORMAL, Color::White, Color::Black, " ");
                        }
                        
                        if grapheme_index == editor.cursor.range.0 {
                            self.rb.print(print_col_num, print_line_num, rustbox::RB_NORMAL, Color::Black, Color::White, " ");
                        }
                    }
                    else {
                        if grapheme_index == editor.cursor.range.0 {
                            self.rb.print(print_col_num, print_line_num, rustbox::RB_NORMAL, Color::Black, Color::White, g);
                        }
                        else {
                            self.rb.print(print_col_num, print_line_num, rustbox::RB_NORMAL, Color::White, Color::Black, g);
                        }
                    }
                    
                    vis_col_num += width;
                    grapheme_index += 1;
                    print_col_num += width;
                    
                    if print_col_num > max_print_col {
                        break;
                    }
                }
            }
            else {
                break;
            }
            
            vis_line_num += 1;
            print_line_num += 1;
            vis_col_num = editor.view_pos.1;
            
            if print_line_num > max_print_line {
                break;
            }
        }
        
        // Print cursor if it's at the end of the text, and thus wasn't printed
        // already.
        if editor.cursor.range.0 >= editor.buffer.grapheme_count() {
            let vis_cursor_pos = editor.buffer.index_to_v2d(editor.cursor.range.0, editor.tab_width);
                if (vis_cursor_pos.0 >= editor.view_pos.0) && (vis_cursor_pos.1 >= editor.view_pos.1) {
                let print_cursor_pos = (vis_cursor_pos.0 - editor.view_pos.0 + c1.0, vis_cursor_pos.1 - editor.view_pos.1 + c1.1);
                
                if print_cursor_pos.0 >= c1.0 && print_cursor_pos.0 <= c2.0 && print_cursor_pos.1 >= c1.1 && print_cursor_pos.1 <= c2.1 {
                    self.rb.print(print_cursor_pos.1, print_cursor_pos.0, rustbox::RB_NORMAL, Color::Black, Color::White, " ");
                }
            }
        }
    }
    
    
}