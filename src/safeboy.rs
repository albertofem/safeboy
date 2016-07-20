extern crate safeboy;
extern crate clap;

use clap::{Arg, App};
use safeboy::frontend::gameboy::Gameboy;

fn main() {
    let matches = App::new("Safeboy")
        .version("1.0")
        .author("Alberto Fernández <albertofem@gmail.com>")
        .about("A GameBoy emulator")
        .arg(Arg::with_name("ROM")
                .help("Rom file to emulate")
                .required(true)
                .index(1)
            )
        .get_matches();

    let rom_file = matches.value_of("ROM").unwrap();

    println!("Welcome to Safeboy! We are preparing your rom to emulate...");
    println!("Loading rom file: {}", rom_file);

    let mut gameboy = Gameboy::new(rom_file);

    gameboy.run();
}