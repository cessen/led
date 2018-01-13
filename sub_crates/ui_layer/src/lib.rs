#[macro_use]
extern crate glium;

use glium::{index, Blend, BlendingFunction, Display, DrawParameters, Frame, IndexBuffer,
            LinearBlendingFactor, Program, Surface, SwapBuffersError, VertexBuffer};

pub struct Window<'a> {
    display: &'a Display,
    solid_color_program: Program,
}

impl<'a> Window<'a> {
    pub fn new(display: &Display) -> Window {
        Window {
            display: display,
            solid_color_program: Program::from_source(
                display,
                VTX_SHADER,
                SOLID_COLOR_SHADER,
                None,
            ).unwrap(),
        }
    }

    pub fn res(&self) -> (u32, u32) {
        let dims = self.display.gl_window().window().get_inner_size().unwrap();
        (dims.0 as u32, dims.1 as u32)
    }

    pub fn draw(&self) -> Drawer {
        let res = self.res();

        Drawer {
            window: &self,
            frame: self.display.draw(),
            resolution: (res.0 as f32, res.1 as f32),
            pixel_space: [
                [2.0 / res.0 as f32, 0.0, 0.0, -1.0],
                [0.0, 2.0 / res.1 as f32, 0.0, -1.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0, 1.0],
            ],
            draw_params: DrawParameters {
                // Premultiplied alpha mode
                blend: Blend {
                    color: BlendingFunction::Addition {
                        source: LinearBlendingFactor::One,
                        destination: LinearBlendingFactor::OneMinusSourceAlpha,
                    },
                    alpha: BlendingFunction::Addition {
                        source: LinearBlendingFactor::One,
                        destination: LinearBlendingFactor::OneMinusSourceAlpha,
                    },
                    constant_value: (0.0, 0.0, 0.0, 0.0),
                },
                // Map clip space to entire viewport
                viewport: Some(glium::Rect {
                    left: 0,
                    bottom: 0,
                    width: res.0,
                    height: res.1,
                }),
                ..Default::default()
            },
        }
    }
}

pub struct Drawer<'a> {
    window: &'a Window<'a>,
    frame: Frame,
    resolution: (f32, f32),
    pixel_space: [[f32; 4]; 4],
    draw_params: DrawParameters<'a>,
}

impl<'a> Drawer<'a> {
    pub fn res(&self) -> (f32, f32) {
        self.resolution
    }

    pub fn clear(&mut self, color: (f32, f32, f32, f32)) {
        self.frame.clear(None, Some(color), true, None, None);
    }

    pub fn rect(&mut self, min: (f32, f32), max: (f32, f32), color: (f32, f32, f32, f32)) {
        let verts = VertexBuffer::new(
            self.window.display,
            &[
                Vert {
                    pos: [min.0, min.1, 0.0, 1.0],
                },
                Vert {
                    pos: [min.0, max.1, 0.0, 1.0],
                },
                Vert {
                    pos: [max.0, max.1, 0.0, 1.0],
                },
                Vert {
                    pos: [max.0, min.1, 0.0, 1.0],
                },
            ],
        ).unwrap();
        let indices = IndexBuffer::new(
            self.window.display,
            index::PrimitiveType::TrianglesList,
            &[0u16, 1, 2, 2, 3, 0],
        ).unwrap();
        let uniforms = uniform! {
            color: [color.0, color.1, color.2, color.3],
            pixel_space: self.pixel_space
        };

        self.frame
            .draw(
                &verts,
                &indices,
                &self.window.solid_color_program,
                &uniforms,
                &self.draw_params,
            )
            .unwrap();
    }

    pub fn finish(self) -> Result<(), SwapBuffersError> {
        self.frame.finish()
    }
}

//===========================================================================

#[derive(Debug, Copy, Clone)]
struct Vert {
    pos: [f32; 4],
}
implement_vertex!(Vert, pos);

const VTX_SHADER: &str = "
#version 330
uniform mat4 pixel_space;
layout(location = 0) in vec4 pos;
void main()
{
    gl_Position = pos * pixel_space;
}
";

const SOLID_COLOR_SHADER: &str = "
#version 330
uniform vec4 color;
out vec4 outputColor;
void main()
{
    outputColor = color;
}
";
