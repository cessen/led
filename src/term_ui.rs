#![allow(dead_code)]

use rustbox;
use rustbox::Color;
use editor::Editor;
use std::char;
use std::time::duration::Duration;


// Key codes
const K_ENTER: u16 = 13;
const K_TAB: u16 = 9;
const K_SPACE: u16 = 32;
const K_BACKSPACE: u16 = 127;
const K_DOWN: u16 = 65516;
const K_LEFT: u16 = 65515;
const K_RIGHT: u16 = 65514;
const K_UP: u16 = 65517;
const K_ESC: u16 = 27;
const K_CTRL_Q: u16 = 17;
const K_CTRL_S: u16 = 19;


pub struct TermUI {
    rb: rustbox::RustBox,
    editor: Editor,
}


impl TermUI {
    pub fn new() -> TermUI {
        TermUI {
            rb: rustbox::RustBox::init(&[None]).unwrap(),
            editor: Editor::new(),
        }
    }
    
    pub fn new_from_editor(editor: Editor) -> TermUI {
        TermUI {
            rb: rustbox::RustBox::init(&[None]).unwrap(),
            editor: editor,
        }
    }
    
    pub fn ui_loop(&mut self) {
        // Quitting flag
        let mut quit = false;
    
        let mut width = self.rb.width();
        let mut height = self.rb.height();
        self.editor.update_dim(height, width);
    
        loop {
            // Draw the editor to screen
            self.rb.clear();
            self.draw_editor(&self.editor, (0, 0), (height-1, width-1));
            self.rb.present();
            
            
            // Handle events.  We block on the first event, so that the
            // program doesn't loop like crazy, but then continue pulling
            // events in a non-blocking way until we run out of events
            // to handle.
            let mut e = self.rb.poll_event(); // Block until we get an event
            loop {
                match e {
                    Ok(rustbox::Event::KeyEvent(_, key, character)) => {
                        //println!("      {} {} {}", modifier, key, character);
                        match key {
                            K_CTRL_Q | K_ESC => {
                                quit = true;
                                break;
                            },
                            
                            K_CTRL_S => {
                                self.editor.save_if_dirty();
                            },
                            
                            K_UP => {
                                self.editor.cursor_up();
                            },
                            
                            K_DOWN => {
                                self.editor.cursor_down();
                            },
                            
                            K_LEFT => {
                                self.editor.cursor_left();
                            },
                            
                            K_RIGHT => {
                                self.editor.cursor_right();
                            },
                            
                            K_ENTER => {
                                self.editor.insert_text_at_cursor("\n");
                            },
                            
                            K_SPACE => {
                                self.editor.insert_text_at_cursor(" ");
                            },
                            
                            K_TAB => {
                                self.editor.insert_text_at_cursor("\t");
                            },
                            
                            K_BACKSPACE => {
                                self.editor.remove_text_behind_cursor(1);
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
                        width = w as uint;
                        height = h as uint;
                        self.editor.update_dim(height, width);
                    }
                    
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

    pub fn draw_editor(&self, editor: &Editor, c1: (uint, uint), c2: (uint, uint)) {
        let mut tb_iter = editor.buffer.iter_at_char(editor.buffer.pos_2d_to_closest_1d(editor.view_pos));
        let mut pline = c1.0;
        let mut pcol = c1.1;
        let mut line = editor.view_pos.0;
        let mut column = editor.view_pos.1;
        let mut pos = editor.buffer.pos_2d_to_closest_1d(editor.view_pos);
        let max_line = line + (c2.0 - c1.0);
        let max_col = column + (c2.1 - c1.1);
        
        let cursor_pos = editor.buffer.pos_2d_to_closest_1d(editor.cursor);
        
        loop {
            if let Option::Some(c) = tb_iter.next() {
                if c == '\n' {
                    if pos == cursor_pos {
                        self.rb.print(pcol, pline, rustbox::RB_NORMAL, Color::Black, Color::White, " ".to_string().as_slice());
                    }
                    
                    pline += 1;
                    pcol = c1.1;
                    line += 1;
                    column = 0;
                }
                else {
                    if pos == cursor_pos  {
                        self.rb.print(pcol, pline, rustbox::RB_NORMAL, Color::Black, Color::White, c.to_string().as_slice());
                    }
                    else {
                        self.rb.print(pcol, pline, rustbox::RB_NORMAL, Color::White, Color::Black, c.to_string().as_slice());
                    }
                    
                    pcol += 1;
                    column += 1;
                }
            }
            else {
                // Show cursor at end of document if it's past the end of
                // the document
                if cursor_pos >= pos {
                    self.rb.print(pcol, pline, rustbox::RB_NORMAL, Color::Black, Color::White, " ");
                }
                
                return;
            }

            if line > max_line {
                return;
            }
            
            // If we're past the edge of the display, go to the next line
            if column > max_col {
                tb_iter.next_line();
                
                pline += 1;
                pcol = c1.1;
                line += 1;
                column = 0;
                
                if line > max_line {
                    return;
                }
            }
            
            // If we're before the edge of the display, move forward to get
            // to it.
            loop {
                if column < editor.view_pos.1 {
                    let nl = tb_iter.skip_non_newline_chars(editor.view_pos.1);
                    if !nl {
                        column = editor.view_pos.1;
                        break;
                    }
                    else {
                        pline += 1;
                        line += 1;
                    }
                    
                    if line > max_line {
                        return;
                    }
                }
                else {
                    break;
                }
            }
            
            // Get the 1d position of the char to be printed next
            pos = editor.buffer.pos_2d_to_closest_1d((line, column));
        }
    }
    
    
}