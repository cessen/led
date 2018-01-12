extern crate clap;
#[macro_use]
extern crate glium;
extern crate unicode_segmentation;
extern crate unicode_width;

use clap::{App, Arg};
use glium::{glutin, index, IndexBuffer, Program, Surface, VertexBuffer};
use glium::draw_parameters::DrawParameters;
use glium::glutin::{Event, KeyboardInput, VirtualKeyCode, WindowEvent};

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

//===========================================================================

#[derive(Debug, Copy, Clone)]
struct Vert {
    pos: [f32; 4],
}
implement_vertex!(Vert, pos);

//===========================================================================

const VTX_SHADER: &str = "
#version 330
layout(location = 0) in vec4 pos;
void main()
{
    gl_Position = pos;
}
";

const FRAG_SHADER: &str = "
#version 330
out vec4 outputColor;
void main()
{
    outputColor = vec4(0.0f, gl_FragCoord.y / 500.0, 1.0f, 1.0f);
}
";

//===========================================================================

fn main() {
    // Parse command line arguments.
    let args = App::new("Led")
        .version(VERSION)
        .about("A humble text editor")
        .arg(
            Arg::with_name("file")
                .value_name("FILE")
                .help("Path to text file to open")
                .required(false),
        )
        .get_matches();

    // Get file path, if specified
    let _filepath = args.value_of("file");

    // Create a window
    let mut events = glutin::EventsLoop::new();
    let display = {
        let window = glutin::WindowBuilder::new()
            .with_title("Hello world!")
            .with_dimensions(512, 512);
        let context = glutin::ContextBuilder::new();
        glium::Display::new(window, context, &events).unwrap()
    };

    // Compile glsl program
    let shader_program = Program::from_source(&display, VTX_SHADER, FRAG_SHADER, None).unwrap();

    // Construct vertex buffer and triangle indices
    let verts = VertexBuffer::new(
        &display,
        &[
            Vert {
                pos: [0.75, 0.75, 0.0, 1.0],
            },
            Vert {
                pos: [0.75, -0.75, 0.0, 1.0],
            },
            Vert {
                pos: [-0.75, -0.75, 0.0, 1.0],
            },
        ],
    ).unwrap();
    let indices =
        IndexBuffer::new(&display, index::PrimitiveType::TrianglesList, &[0u16, 1, 2]).unwrap();

    // Event loop
    let mut stop = false;
    while !stop {
        // Process events
        events.poll_events(|e| match e {
            Event::WindowEvent {
                event: WindowEvent::Closed,
                ..
            } => {
                stop = true;
            }
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    },
                ..
            } => {
                stop = true;
            }
            _ => {}
        });

        // Render
        let mut frame = display.draw();
        frame.clear(None, Some((0.18, 0.18, 0.18, 1.0)), true, None, None);
        frame
            .draw(
                &verts,
                &indices,
                &shader_program,
                &glium::uniforms::EmptyUniforms,
                &DrawParameters {
                    viewport: Some(glium::Rect {
                        left: 0,
                        bottom: 0,
                        width: 512,
                        height: 512,
                    }),
                    ..Default::default()
                },
            )
            .unwrap();
        frame.finish().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}
