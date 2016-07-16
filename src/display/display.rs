extern crate glium;

const WIDTH: u32 = 160;
const HEIGHT: u32 = 144;

use self::glium::{DisplayBuild, Surface};
use self::glium::backend::glutin_backend::GlutinFacade;
use self::glium::texture::texture2d::Texture2d;
use self::glium::texture::RawImage2d;
use self::glium::glutin::VirtualKeyCode;
use self::glium::glutin::ElementState::{Pressed, Released};
use std::borrow::Cow;

pub enum EventType {
    None,
    Pressed,
    Released
}

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
    Right
}

pub struct Display {
    glium_display: Option<GlutinFacade>,
    screen: Option<Texture2d>
}

impl Display {
    pub fn new() -> Display {
        Display {
            glium_display: None,
            screen: None
        }
    }

    pub fn initialize(&mut self) -> () {
        self.glium_display = Some(glium::glutin::WindowBuilder::new()
            .with_dimensions(WIDTH, HEIGHT)
            .with_title(format!("Safeboy"))
            .build_glium()
            .unwrap()
        );

        self.screen = Some(Texture2d::empty_with_format(
                self.glium_display.as_mut().unwrap(),
                glium::texture::UncompressedFloatFormat::U8U8U8,
                glium::texture::MipmapsOption::NoMipmap,
                WIDTH as u32,
                HEIGHT as u32
            ).unwrap()
        );

        self.reset();
    }

    pub fn poll_events(&mut self) -> (EventType, Event) {
        for event in self.glium_display.as_mut().unwrap().poll_events() {
            return match event {
                glium::glutin::Event::Closed => {
                    (EventType::None, Event::Closed)
                },
                glium::glutin::Event::KeyboardInput(Pressed, _, Some(pressed_key)) => {
                    (EventType::Pressed, Display::map_events(pressed_key).unwrap())
                },
                glium::glutin::Event::KeyboardInput(Released, _, Some(released_key)) => {
                    (EventType::Released, Display::map_events(released_key).unwrap())
                }
                _ => (EventType::None, Event::None)
            };
        }

        (EventType::None, Event::None)
    }

    fn map_events(glutin_key: VirtualKeyCode) -> Result<Event, &'static str> {
        match glutin_key {
            VirtualKeyCode::Return => {
                Ok(Event::Start)
            },
            VirtualKeyCode::Back => {
                Ok(Event::Select)
            }
            VirtualKeyCode::Z => {
                Ok(Event::A)
            },
            VirtualKeyCode::X => {
                Ok(Event::B)
            },
            VirtualKeyCode::Up => {
                Ok(Event::Up)
            },
            VirtualKeyCode::Down => {
                Ok(Event::Down)
            },
            VirtualKeyCode::Left => {
                Ok(Event::Left)
            },
            VirtualKeyCode::Right => {
                Ok(Event::Right)
            },
            _ => Ok(Event::Unknown)
        }
    }

    pub fn draw(&mut self, raw_pixels: &[u8]) {
        let raw_image = RawImage2d {
            data: Cow::Borrowed(raw_pixels),
            width: WIDTH,
            height: HEIGHT,
            format: glium::texture::ClientFormat::U8U8U8
        };

        self.screen.as_mut().unwrap().write(
            glium::Rect {
                left: 0,
                bottom: 0,
                width: WIDTH as u32,
                height: HEIGHT as u32
            },
            raw_image
        );

        let target = self.glium_display.as_mut().unwrap().draw();

        self.screen.as_mut().unwrap().as_surface().blit_whole_color_to(
            &target,
            &glium::BlitTarget {
                left: 0,
                bottom: HEIGHT,
                width: WIDTH as i32,
                height: -(HEIGHT as i32)
            },
            glium::uniforms::MagnifySamplerFilter::Linear
        );

        target.finish().unwrap();
    }

    pub fn reset(&mut self) {
        let mut target = self.glium_display.as_mut().unwrap().draw();

        target.clear_color(0.0, 0.0, 0.0, 1.0);
        target.finish().unwrap();
    }
}