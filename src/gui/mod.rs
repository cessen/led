#![allow(dead_code)]

use sdl2;
use sdl2::event::WindowEventId;
use sdl2::render::{Renderer, Texture};
use sdl2::rect::Rect;

use font::Font;
use editor::Editor;
use self::formatter::GUILineFormatter;

pub mod formatter;

pub struct GUI {
    renderer: Renderer,
    draw_buf: Texture,
    font: Font,
    editor: Editor<GUILineFormatter>,
}

impl GUI {
    pub fn new() -> GUI {
        let font = Font::new_default(14);
        
        // Get the window and renderer for sdl
        let window = sdl2::video::Window::new("Led Editor", sdl2::video::WindowPos::PosCentered, sdl2::video::WindowPos::PosCentered, 800, 600, sdl2::video::OPENGL | sdl2::video::RESIZABLE).unwrap();
        let renderer = sdl2::render::Renderer::from_window(window, sdl2::render::RenderDriverIndex::Auto, sdl2::render::ACCELERATED).unwrap();
        let draw_buf = renderer.create_texture(sdl2::pixels::PixelFormatFlag::RGBA8888, sdl2::render::TextureAccess::Target, 1, 1).unwrap();
        
        let mut editor = Editor::new(GUILineFormatter::new(4));
        editor.update_dim(renderer.get_output_size().unwrap().1 as usize, renderer.get_output_size().unwrap().0 as usize);
        
        GUI {
            renderer: renderer,
            draw_buf: draw_buf,
            font: font,
            editor: editor,
        }
    }
    

    pub fn new_from_editor(ed: Editor<GUILineFormatter>) -> GUI {
        let font = Font::new_default(14);
        
        // Get the window and renderer for sdl
        let window = sdl2::video::Window::new("Led Editor", sdl2::video::WindowPos::PosCentered, sdl2::video::WindowPos::PosCentered, 800, 600, sdl2::video::OPENGL | sdl2::video::RESIZABLE).unwrap();
        let renderer = sdl2::render::Renderer::from_window(window, sdl2::render::RenderDriverIndex::Auto, sdl2::render::ACCELERATED).unwrap();
        let draw_buf = renderer.create_texture(sdl2::pixels::PixelFormatFlag::RGBA8888, sdl2::render::TextureAccess::Target, 1, 1).unwrap();
        
        let mut editor = ed;
        editor.update_dim(renderer.get_output_size().unwrap().1 as usize, renderer.get_output_size().unwrap().0 as usize);
        
        GUI {
            renderer: renderer,
            draw_buf: draw_buf,
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
                            break;
                        }
                    },
                    
                    sdl2::event::Event::Window(_, _, WindowEventId::Exposed, _, _) => {
                        // Remove other Exposed events from the queue, to avoid
                        // clogging things up.
                        sdl2::event::filter_events(filter_window_expose);
                        
                        // Get renderer size
                        let (w, h) = self.renderer.get_output_size().unwrap();
                        
                        // Check if we should re-render the UI before blitting
                        // it over.
                        let redraw = match self.draw_buf.query() {
                            Ok(tq) => tq.width != w || tq.height != h,
                            _ => true,
                        };
                        
                        if redraw {
                            // Realloc texture to match renderer size
                            self.draw_buf = self.renderer.create_texture(sdl2::pixels::PixelFormatFlag::RGBA8888, sdl2::render::TextureAccess::Target, w, h).unwrap();
                        
                            // Draw UI to texture
                            let _ = self.renderer.set_render_target(Some(&self.draw_buf));
                            let _ = self.renderer.set_draw_color(sdl2::pixels::Color::RGB(80, 80, 80));
                            let _ = self.renderer.clear();
                            self.draw_editor_text((50, 50), (300, 300));
                        }
                        
                        // Blit texture over
                        let _ = self.renderer.set_render_target(None);
                        let _ = self.renderer.copy(&self.draw_buf, Some(Rect{x:0, y:0, h:h as i32, w:w as i32}), Some(Rect{x:0, y:0, h:h as i32, w:w as i32}));
                        self.renderer.present();
                    },
                    
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
                x += self.font.draw_text(g, (255, 255, 255), x as i32, y as i32, &self.renderer);
            }
            
            x = c1.1;
            y += self.font.line_height() >> 6;
        }
    }
}


// Used for removing WindowEventId::Exposed events from the queue
extern "C" fn filter_window_expose(e: sdl2::event::Event) -> bool {
    match e {
        sdl2::event::Event::Window(_, _, WindowEventId::Exposed, _, _) => false,
        _ => true,
    }
}