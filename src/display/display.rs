extern crate glium;

const WIDTH: u32 = 160;
const HEIGHT: u32 = 144;

use self::glium::{DisplayBuild, Surface};
use self::glium::backend::glutin_backend::GlutinFacade;

pub enum Event {
    None,
    Closed
}

pub struct Display {
    gliumDisplay: Option<GlutinFacade>
}

impl Display {
    pub fn new() -> Display {
        Display {
            gliumDisplay: None
        }
    }

    pub fn initialize(&mut self, game_name: &str) -> () {
        self.gliumDisplay = Some(glium::glutin::WindowBuilder::new()
            .with_dimensions(WIDTH, HEIGHT)
            .with_title(format!("Safeboy: {:?}", game_name))
            .build_glium()
            .unwrap());
    }

    pub fn poll_events(&mut self) -> Event {
        for event in self.gliumDisplay.as_mut().unwrap().poll_events() {
            match event {
                glium::glutin::Event::Closed => return Event::Closed,
                _ => return Event::None
            }
        }

        Event::None
    }

    pub fn draw(&mut self) {
        let mut target = self.gliumDisplay.as_mut().unwrap().draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);
        target.finish().unwrap();
    }
}