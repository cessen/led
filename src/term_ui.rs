use rustbox;
use rustbox::{Style,Color};
use editor::Editor;

pub fn draw_editor(editor: &Editor, c1: (uint, uint), c2: (uint, uint)) {
    let mut tb_iter = editor.buffer.root_iter();
    let mut line: uint = 0;
    let mut column: uint = 0;
    let height = c2.0 - c1.0;
    let width = c2.1 - c1.1;
    
    loop {
        if let Option::Some(c) = tb_iter.next() {
            if c == '\n' {
                if editor.cursor.0 == line && editor.cursor.1 >= column && editor.cursor.1 <= width {
                    rustbox::print(editor.cursor.1, line, Style::Normal, Color::Black, Color::White, " ".to_string());
                }
                
                line += 1;
                column = 0;
                continue;
            }
            
            if editor.cursor.0 == line && editor.cursor.1 == column  {
                rustbox::print(column, line, Style::Normal, Color::Black, Color::White, c.to_string());
            }
            else {
                rustbox::print(column, line, Style::Normal, Color::White, Color::Black, c.to_string());
            }
            column += 1;
        }
        else {
            break;
        }
        
        if line > height {
            break;
        }
        
        if column > width {
            tb_iter.next_line();
            line += 1;
            column = 0;
        }
    }
    
    if editor.cursor.0 == line && editor.cursor.1 >= column && editor.cursor.1 <= width {
        rustbox::print(editor.cursor.1, line, Style::Normal, Color::Black, Color::White, " ".to_string());
    }
    else if editor.cursor.0 > line && editor.cursor.0 <= height && editor.cursor.1 <= width {
        rustbox::print(editor.cursor.1, editor.cursor.0, Style::Normal, Color::Black, Color::White, " ".to_string());
    }
}