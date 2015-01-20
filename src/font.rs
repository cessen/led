#![allow(dead_code)]

use std::path::Path;
use std::collections::HashMap;

use freetype;
use sdl2;

use sdl2::surface::Surface;
use sdl2::rect::Rect;

use string_utils::{is_line_ending};

struct CachedGlyph {
    texture: Option<sdl2::render::Texture>,
    height: i32,
    width: i32,
    advance: i32,
    bitmap_top: i32,
    bitmap_left: i32,
}


pub struct Font {
    ftl: freetype::Library,
    face: freetype::Face,
    glyph_cache: HashMap<char, CachedGlyph>,
}

impl Font {
    pub fn new_from_file(path: &Path, size: u32) -> Font {
        let lib = freetype::Library::init().unwrap();
        let mut face = lib.new_face(path.as_str().unwrap(), 0).unwrap();
        let _ = face.set_pixel_sizes(0, size);
        
        Font {
            ftl: lib,
            face: face,
            glyph_cache: HashMap::new(),
        }
    }
    
    
    pub fn line_height(&self) -> usize {
        self.face.height() as usize
    }
    

    pub fn draw_text(&mut self, text: &str, color: (u8, u8, u8), cx: i32, cy: i32, renderer: &sdl2::render::Renderer) -> usize {
        let mut x = cx;
        let y = cy;
        
        for grapheme in text.graphemes(true) {
            if is_line_ending(grapheme) {
                continue;
            }
            else if grapheme == "\t" {
                // TODO: handle tab characters
            }
            else {
                let ch = grapheme.chars().next().unwrap();

                // Generate and cache glyph if we haven't already
                if !self.glyph_cache.contains_key(&ch) {
                    let mut cg = CachedGlyph {
                        texture: None,
                        height: 0,
                        width: 0,
                        advance: 0,
                        bitmap_top: 0,
                        bitmap_left: 0,
                    };
                    
                    
                    let _ = self.face.load_char(ch as u64, freetype::face::RENDER);
                    let g = self.face.glyph();
                
                    match (g.bitmap().width(), g.bitmap().rows()) {
                        (0, _) | (_, 0) => {
                            cg.advance = (g.advance().x >> 6) as i32;
                        },
                        
                        _ => {
                            // Get the char's glyph bitmap as an sdl surface
                            let bitmap = g.bitmap();
                            cg.width = g.bitmap().width();
                            cg.height = g.bitmap().rows();
                            cg.advance = (g.advance().x >> 6) as i32;
                            cg.bitmap_left = g.bitmap_left();
                            cg.bitmap_top = g.bitmap_top();
                            
                            let mut buf = Vec::with_capacity(bitmap.buffer().len() * 4);
                            for b in bitmap.buffer().iter() {
                                buf.push(*b);
                                buf.push(color.2);
                                buf.push(color.1);
                                buf.push(color.0);
                            }
                            let gs = Surface::from_data(buf.as_mut_slice(), cg.width as isize, cg.height as isize, 32, (cg.width as isize) * 4, 0xFF000000, 0x00FF0000, 0x0000FF00, 0x000000FF).unwrap();
                            
                            // Get glyph surface as a texture
                            cg.texture = Some(renderer.create_texture_from_surface(&gs).unwrap());
                        }
                    }
                    
                    self.glyph_cache.insert(ch, cg);
                }

                // Draw the glyph
                let ref cg = self.glyph_cache[ch];
                if let Some(ref tex) = cg.texture {
                    let _ = renderer.copy(tex, Some(Rect{x:0, y:0, h:cg.height, w:cg.width}), Some(Rect{x:x+cg.bitmap_left, y:y-cg.bitmap_top, h:cg.height, w:cg.width}));
                }
                
                x += cg.advance;
            }
        }
        
        return (x - cx) as usize;
    }
}
