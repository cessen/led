#![allow(dead_code)]

use std::path::Path;

use freetype;
use sdl2;

use sdl2::surface::Surface;
use sdl2::rect::Rect;


pub struct Font {
    ftl: freetype::Library,
    face: freetype::Face,
}

impl Font {
    pub fn new_from_file(path: &Path, size: u32) -> Font {
        let lib = freetype::Library::init().unwrap();
        let mut face = lib.new_face(path.as_str().unwrap(), 0).unwrap();
        let _ = face.set_pixel_sizes(0, size);
        
        Font {
            ftl: lib,
            face: face,
        }
    }
    
    
    pub fn draw_text(&mut self, text: &str, color: (u8, u8, u8), cx: i32, cy: i32, renderer: &sdl2::render::Renderer) {
        let mut x = cx;
        let mut y = cy;
        
        for grapheme in text.graphemes(true) {
            for ch in grapheme.chars() {
                let _ = self.face.load_char(ch as u64, freetype::face::RENDER);
                let g = self.face.glyph();
            
                match (g.bitmap().width(), g.bitmap().rows()) {
                    (0, _) | (_, 0) => {
                    },
                    
                    _ => {
                        // Get the char's glyph bitmap as an sdl surface
                        
                        let bitmap = g.bitmap();
                        let width = g.bitmap().width() as isize;
                        let height = g.bitmap().rows() as isize;
                        let mut buf = Vec::with_capacity(bitmap.buffer().len() * 4);
                        for b in bitmap.buffer().iter() {
                            buf.push(*b);
                            buf.push(color.2);
                            buf.push(color.1);
                            buf.push(color.0);
                        }
                        let gs = Surface::from_data(buf.as_mut_slice(), width, height, 32, width*4, 0xFF000000, 0x00FF0000, 0x0000FF00, 0x000000FF).unwrap();
                        
                        // Get glyph surface as a texture
                        let gt = renderer.create_texture_from_surface(&gs).unwrap();
                        
                        // Draw the glyph
                        let _ = renderer.copy(&gt, Some(Rect{x:0, y:0, h:height as i32, w:width as i32}), Some(Rect{x:x+g.bitmap_left(), y:y-g.bitmap_top(), h:height as i32, w:width as i32}));
                        
                        
                    }
                }
                
                x += (g.advance().x >> 6) as i32;
                y += (g.advance().y >> 6) as i32;
                break;
            }
        }
    }
}
