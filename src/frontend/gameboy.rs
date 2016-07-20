use cpu::z80::Z80;
use display::display::{Display, Event, EventType};
use frontend::keypad::Key;

/// GameBoy
///
/// This is the main entry point to run GameBoy games.
/// It contains the CPU and the OpenGL display
pub struct Gameboy {
    cpu: Z80,
    display: Display
}

/// Basic signals
///
/// These signals are used to process non-key events,
/// like closing the window
#[derive(PartialEq, Copy, Clone)]
enum EventSignal {
    None,
    Close
}

impl Gameboy {
    /// Creates a new GameBoy instance
    ///
    /// We need the GameBoy (.gb) file that will be run
    pub fn new(rom_file: &str) -> Gameboy {
        Gameboy {
            cpu: Z80::new(rom_file),
            display: Display::new(),
        }
    }

    /// Runs the game
    ///
    /// This will enter the main loop and process
    /// CPU, MMU, GPU cycles and finally draw them
    /// to the OpenGL display.
    ///
    /// It will also poll for events (keyboard) and translate
    /// them into GameBoy-valid keypad events
    pub fn run(&mut self) -> () {
        self.display.initialize();

        loop {
            if self.poll_events() == EventSignal::Close {
                break;
            }

            self.cpu.step();
            self.display.draw(self.cpu.get_gpu_pixels());
        }
    }

    fn poll_events(&mut self) -> EventSignal
    {
        let signal = match self.display.poll_events() {
            (_, Event::Closed) => {
                println!("Closing Gameboy, safe travels!");
                EventSignal::Close
            },
            (_, Event::Unknown) => {
                EventSignal::None
            },
            (EventType::Pressed, pressed_key) => {
                let key = Gameboy::map_events_to_keypad(pressed_key);
                self.cpu.key_down(key);
                EventSignal::None
            },
            (EventType::Released, released_key) => {
                let key = Gameboy::map_events_to_keypad(released_key);
                self.cpu.key_up(key);
                EventSignal::None
            }
            _ => EventSignal::None
        };

        return signal;
    }

    fn map_events_to_keypad(event: Event) -> Key {
        match event {
            Event::Start => Key::Start,
            Event::Select => Key::Select,
            Event::A => Key::A,
            Event::B => Key::B,
            Event::Up => Key::Up,
            Event::Down => Key::Down,
            Event::Left => Key::Left,
            Event::Right => Key::Right,
            _ => panic!("Unknown key pressed")
        }
    }
}