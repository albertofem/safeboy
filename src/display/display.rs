extern crate glium;

const WIDTH: u32 = 160;
const HEIGHT: u32 = 144;

use self::glium::{DisplayBuild, Surface};
use self::glium::backend::glutin_backend::GlutinFacade;

pub struct Display {
    gliumDisplay: GlutinFacade
}

impl Display {
    pub fn new(rom_file: &str) -> Display {
        let display = glium::glutin::WindowBuilder::new()
            .with_dimensions(WIDTH, HEIGHT)
            .with_title(format!("Safeboy: {:?}", rom_file))
            .build_glium()
            .unwrap();

        Display {
            gliumDisplay: display
        }
    }

    pub fn run(&mut self) {
        loop {
            let mut target = self.gliumDisplay.draw();
            target.clear_color(0.0, 0.0, 0.0, 1.0);
            target.finish().unwrap();

            for ev in self.gliumDisplay.poll_events() {
                match ev {
                    glium::glutin::Event::Closed => return,
                    _ => ()
                }
            }
        }
    }
}