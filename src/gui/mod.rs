#![allow(dead_code)]

use sdl2;

use font::Font;
use editor::Editor;


pub struct GUI {
    renderer: sdl2::render::Renderer,
    font: Font,
    editor: Editor,
}

impl GUI {
    pub fn new() -> GUI {
        let font = Font::new_from_file(&Path::new("./fonts/source_code_pro/SourceCodePro-Regular.ttf"), 14);
        
        // Get the window and renderer for sdl
        let window = sdl2::video::Window::new("Led Editor", sdl2::video::WindowPos::PosCentered, sdl2::video::WindowPos::PosCentered, 800, 600, sdl2::video::OPENGL | sdl2::video::RESIZABLE).unwrap();
        let renderer = sdl2::render::Renderer::from_window(window, sdl2::render::RenderDriverIndex::Auto, sdl2::render::ACCELERATED).unwrap();
        
        let mut editor = Editor::new();
        editor.update_dim(renderer.get_output_size().unwrap().1 as usize, renderer.get_output_size().unwrap().0 as usize);
        
        GUI {
            renderer: renderer,
            font: font,
            editor: editor,
        }
    }
    

    pub fn new_from_editor(ed: Editor) -> GUI {
        let font = Font::new_from_file(&Path::new("./fonts/source_code_pro/SourceCodePro-Regular.ttf"), 14);
        
        // Get the window and renderer for sdl
        let window = sdl2::video::Window::new("Led Editor", sdl2::video::WindowPos::PosCentered, sdl2::video::WindowPos::PosCentered, 800, 600, sdl2::video::OPENGL | sdl2::video::RESIZABLE).unwrap();
        let renderer = sdl2::render::Renderer::from_window(window, sdl2::render::RenderDriverIndex::Auto, sdl2::render::ACCELERATED).unwrap();
        
        let mut editor = ed;
        editor.update_dim(renderer.get_output_size().unwrap().1 as usize, renderer.get_output_size().unwrap().0 as usize);
        
        GUI {
            renderer: renderer,
            font: font,
            editor: editor,
        }
    }
    
    pub fn main_ui_loop(&mut self) {
        loop {
            if let Ok(e) = sdl2::event::wait_event() {
                match e {
                    sdl2::event::Event::Quit(_) => break,
                    sdl2::event::Event::KeyDown(_, _, key, _, _, _) => {
                        if key == sdl2::keycode::KeyCode::Escape {
                            break
                        }
                    },
                    sdl2::event::Event::Window(_, _, sdl2::event::WindowEventId::Exposed, _, _) => {
                        let _ = self.renderer.set_draw_color(sdl2::pixels::Color::RGB(240, 240, 240));
                        let _ = self.renderer.clear();
                        //self.font.draw_text("Hi there!  (How's it going???) { let b = 42; }", (0, 0, 0), 50, 50, &self.renderer);
                        self.draw_editor_text((50, 50), (300, 300));
                        self.renderer.present();
                    }
                    _ => {}
                }
            }
        }
    }
    
    fn draw_editor_text(&mut self, c1: (usize, usize), c2: (usize, usize)) {
        let mut line_iter = self.editor.buffer.line_iter();
        
        let mut x = c1.1;
        let mut y = c1.0;
        
        for line in line_iter {
            for g in line.grapheme_iter() {
                x += self.font.draw_text(g, (0, 0, 0), x as i32, y as i32, &self.renderer);
            }
            
            x = c1.1;
            y += self.font.line_height() >> 6;
        }
    }
}