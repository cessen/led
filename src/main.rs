extern crate clap;
extern crate glium;
extern crate ui_layer;
extern crate unicode_segmentation;
extern crate unicode_width;

use ui_layer::Window;
use clap::{App, Arg};
use glium::glutin;
use glium::glutin::{Event, KeyboardInput, VirtualKeyCode, WindowEvent};

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

const default_font: &[u8] = include_bytes!("font/SourceCodePro-Regular.ttf");

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
    let window = Window::new(&display);

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
        let mut frame = window.draw();
        let res = frame.res();
        frame.clear((0.18, 0.18, 0.18, 1.0));
        frame.rect((0.0, 0.0), (32.0, 64.0), (0.8, 0.8, 0.1, 1.0));
        frame.rect((0.0, res.1 - 64.0), (32.0, res.1), (0.8, 0.1, 0.8, 1.0));
        frame.finish().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}
