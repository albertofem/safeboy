extern crate glium;

const WIDTH: u32 = 160;
const HEIGHT: u32 = 144;

use self::glium::Surface;
use self::glium::backend::glutin::Display as GliumDisplay;
use self::glium::glutin::EventsLoop;
use self::glium::texture::texture2d::Texture2d;
use self::glium::texture::RawImage2d;
use self::glium::glutin::VirtualKeyCode;
use self::glium::glutin::ElementState::{Pressed, Released};
use self::glium::glutin;
use std::borrow::Cow;

/// Type of event
///
/// Whether a button was pressed or not .This is just
/// to map the window events (as processed by Glium)
/// to GameBoy events
pub enum EventType {
    None,
    Pressed,
    Released,
}

/// Event enum
///
/// This contains the GameBoy events that will be later
/// translated to Keypad keys and processed by the keypad
/// module
pub enum Event {
    Unknown,
    None,
    Closed,
    Start,
    Select,
    A,
    B,
    Up,
    Down,
    Left,
    Right,
}

/// Display struct
///
/// It contains the Glutin display (which is the window)
/// and the screen (a 2d texture where pixels are drawn)
pub struct Display {
    event_loop: Option<EventsLoop>,
    glium_display: Option<GliumDisplay>,
    screen: Option<Texture2d>,
}

impl Display {
    pub fn new() -> Display {
        Display {
            event_loop: None,
            glium_display: None,
            screen: None,
        }
    }

    /// Initialize the display
    ///
    /// We create a Glium window with the GameBoy dimensions and
    /// also de 2d texture (with same dimensions)
    pub fn initialize(&mut self) -> () {
        self.event_loop = Some(EventsLoop::new());

        let window = glium::glutin::WindowBuilder::new()
            .with_dimensions(WIDTH, HEIGHT)
            .with_title(format!("Safeboy"));

        let context = glium::glutin::ContextBuilder::new()
            .with_vsync(true);

        self.glium_display = Some(GliumDisplay::new(
            window,
            context,
            self.event_loop.as_mut().unwrap(),
        ).unwrap());

        self.screen = Some(Texture2d::empty_with_format(
            self.glium_display.as_mut().unwrap(),
            glium::texture::UncompressedFloatFormat::U8U8U8,
            // we don't need mipmap as we are working with raw bytes
            glium::texture::MipmapsOption::NoMipmap,
            WIDTH as u32,
            HEIGHT as u32,
        ).unwrap()
        );

        self.reset();
    }

    /// Poll events
    ///
    /// This will return a tuple containing the EventType
    /// and the Event ocurred in the Window system. This is
    /// mainly to capture key presses to be later converted
    /// to GameBoy understandable keys
    pub fn poll_events(&mut self) -> (EventType, Event) {
        let mut event_type = EventType::None;
        let mut event_triggered = Event::None;

        self.event_loop.as_mut().unwrap().poll_events(|event| {
            match event {
                glutin::Event::WindowEvent { event, .. } => match event {
                    glutin::WindowEvent::Closed => {
                        event_type = EventType::None;
                        event_triggered = Event::Closed;
                    }
                    glutin::WindowEvent::KeyboardInput { input, .. } => match input.state {
                        Pressed => {
                            event_type = EventType::Pressed;
                            event_triggered = Display::map_events(input.virtual_keycode.unwrap()).unwrap();
                        }
                        Released => {
                            event_type = EventType::Released;
                            event_triggered = Display::map_events(input.virtual_keycode.unwrap()).unwrap();
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        });

        return (event_type, event_triggered);
    }

    fn map_events(glutin_key: VirtualKeyCode) -> Result<Event, &'static str> {
        match glutin_key {
            VirtualKeyCode::Return => {
                Ok(Event::Start)
            }
            VirtualKeyCode::Back => {
                Ok(Event::Select)
            }
            VirtualKeyCode::Z => {
                Ok(Event::A)
            }
            VirtualKeyCode::X => {
                Ok(Event::B)
            }
            VirtualKeyCode::Up => {
                Ok(Event::Up)
            }
            VirtualKeyCode::Down => {
                Ok(Event::Down)
            }
            VirtualKeyCode::Left => {
                Ok(Event::Left)
            }
            VirtualKeyCode::Right => {
                Ok(Event::Right)
            }
            _ => Ok(Event::Unknown)
        }
    }

    /// Draws the screen
    ///
    /// This method will draw all raw pixels in the screen
    /// using OpenGL primitives. More info in the method
    /// implementation.
    pub fn draw(&mut self, raw_pixels: &[u8]) {

        // create a raw 2d image with pixels coming
        // from the GPU. From Glium docs:
        // The data must start by the bottom-left hand corner pixel and progress left-to-right and bottom-to-top.
        // As our pixel data is not this way, we will later need to perform a correction
        // in order to draw in the OpenGL context
        let raw_image = RawImage2d {
            data: Cow::Borrowed(raw_pixels),
            width: WIDTH,
            height: HEIGHT,
            // each pixel is represented with three components (RGB)
            // this flag tells OpenGL to read it in this format
            format: glium::texture::ClientFormat::U8U8U8,
        };

        // write the raw image to the 2d texture buffer
        // starting bottom leff for the display width and height
        self.screen.as_mut().unwrap().write(
            glium::Rect {
                left: 0,
                bottom: 0,
                width: WIDTH as u32,
                height: HEIGHT as u32,
            },
            raw_image,
        );

        // select the target from our display
        let target = self.glium_display.as_mut().unwrap().draw();

        // paste texture in our OpenGL context
        // we need to convert our generated from top left pixel array
        // to OpenGL's coordinate system (where Y is going from bottom to top)
        self.screen.as_mut().unwrap().as_surface().blit_whole_color_to(
            &target,
            &glium::BlitTarget {
                left: 0,
                bottom: HEIGHT,
                width: WIDTH as i32,
                height: -(HEIGHT as i32), // invert vertical
            },
            glium::uniforms::MagnifySamplerFilter::Linear, // what to in case
        );

        target.finish().unwrap();
    }

    pub fn reset(&mut self) {
        let mut target = self.glium_display.as_mut().unwrap().draw();

        target.clear_color(0.0, 0.0, 0.0, 1.0);
        target.finish().unwrap();
    }
}